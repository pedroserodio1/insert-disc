//! Cenários de docs/TESTING-WITH-ISO.md rodando a máquina de estados inteira no drive falso.

use std::fs;
use std::path::PathBuf;

use insert_disc_core::app::*;
use insert_disc_core::catalog::*;
use insert_disc_core::classify::MediaClass;
use insert_disc_core::drive::*;
use insert_disc_core::fake::FakeIsoDrive;
use insert_disc_core::gameini;
use insert_disc_core::iso;
use insert_disc_core::launch::{LaunchError, LaunchRequest, Launcher};
use tempfile::TempDir;
use uuid::Uuid;

#[derive(Default)]
struct Rec {
    calls: Vec<LaunchRequest>,
    fail: Option<LaunchError>,
}

impl Launcher for Rec {
    fn launch(&mut self, req: &LaunchRequest) -> Result<(), LaunchError> {
        if let Some(e) = self.fail.clone() {
            return Err(e);
        }
        self.calls.push(req.clone());
        Ok(())
    }
}

type TestApp = App<FakeIsoDrive, Rec>;

struct Env {
    dir: TempDir,
    app: TestApp,
    x: GameId,
    y: GameId,
    x_iso: PathBuf,
    y_iso: PathBuf,
    x_disc: DiscId,
}

fn write_iso(dir: &TempDir, name: &str, ini: &str) -> PathBuf {
    let p = dir.path().join(format!("{name}.iso"));
    fs::write(&p, iso::build_iso(&gameini::sanitize_label(name), &[("GAME.INI", ini.as_bytes())])).unwrap();
    p
}

fn disc(id: DiscId) -> Disc {
    Disc { disc_id: id, label: "L".into(), media: DiscMedia::CdR, created_at: 1, origin: DiscOrigin::Burned, note: None }
}

fn catalog(x_disc: DiscId, y_disc: DiscId) -> (Catalog, GameId, GameId) {
    let mut c = Catalog::default();
    let x = c.add_game(Game::new("Hollow Knight", GameKind::Steam { app_id: 367520 })).unwrap();
    let y = c.add_game(Game::new("Celeste", GameKind::Steam { app_id: 504230 })).unwrap();
    c.add_disc(x, disc(x_disc)).unwrap();
    c.add_disc(y, disc(y_disc)).unwrap();
    (c, x, y)
}

fn env_with(prep: impl FnOnce(&mut FakeIsoDrive, &PathBuf, &PathBuf)) -> Env {
    let dir = tempfile::tempdir().unwrap();
    let (xd, yd) = (Uuid::new_v4(), Uuid::new_v4());
    let x_iso = write_iso(&dir, "hk", &gameini::write(xd, "Hollow Knight"));
    let y_iso = write_iso(&dir, "celeste", &gameini::write(yd, "Celeste"));
    let (c, x, y) = catalog(xd, yd);
    let mut drive = FakeIsoDrive::new(dir.path().join("burned"));
    prep(&mut drive, &x_iso, &y_iso);
    let app = App::boot(Ok(c), None, drive, Rec::default(), 0);
    Env { dir, app, x, y, x_iso, y_iso, x_disc: xd }
}

fn env() -> Env {
    env_with(|_, _, _| {})
}

impl Env {
    fn select(&mut self, g: GameId, t: u64) {
        self.app.dispatch(Intent::Select(g), t).unwrap();
    }
    fn act(&mut self, a: Action, t: u64) {
        self.app.dispatch(Intent::Action(a), t).unwrap();
    }
    fn name(&self) -> &'static str {
        self.app.state().name()
    }
    fn launches(&self) -> Vec<LaunchRequest> {
        self.app.launcher().calls.clone()
    }
    fn game_iso(&self, name: &str, id: Uuid) -> PathBuf {
        write_iso(&self.dir, name, &gameini::write(id, name))
    }
}

fn steam(id: u32) -> LaunchRequest {
    LaunchRequest::Uri(format!("steam://run/{id}"))
}

// ---------- jogar ----------

#[test]
fn c1_selected_game_with_its_disc_launches_after_the_sheet_and_the_loading_minimum() {
    let mut e = env();
    e.select(e.x, 0);
    assert_eq!(e.app.state(), &State::WaitingDisc { game: e.x, tray: TrayHint::Opening });
    let iso = e.x_iso.clone();
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(10);
    assert_eq!(e.name(), "IDENTIFIED");
    assert!(e.launches().is_empty(), "a ficha aparece antes de lançar");
    e.app.pump(10 + FICHA_MIN_MS);
    assert_eq!(e.name(), "LAUNCHING");
    assert_eq!(e.launches(), [steam(367520)]);
    assert!(!e.app.armed());
    e.app.pump(10 + FICHA_MIN_MS + 2999);
    assert_eq!(e.name(), "LAUNCHING", "duração mínima (loading_min_ms = 3000)");
    e.app.pump(10 + FICHA_MIN_MS + 3000);
    assert_eq!(e.app.state(), &State::Library);
    assert_eq!(e.app.dispatch(Intent::Back, 0), Err(IntentError::Ignored));
}

#[test]
fn loading_minimum_zero_finishes_on_the_next_pump() {
    let mut e = env();
    e.app.dispatch(Intent::OpenSettings, 0).unwrap();
    e.app.dispatch(Intent::SetSetting(SettingChange::LoadingMinMs(0)), 0).unwrap();
    e.app.dispatch(Intent::Back, 0).unwrap();
    e.select(e.x, 0);
    let iso = e.x_iso.clone();
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(0);
    e.app.pump(FICHA_MIN_MS);
    assert_eq!(e.name(), "LAUNCHING");
    e.app.pump(FICHA_MIN_MS);
    assert_eq!(e.app.state(), &State::Library);
}

#[test]
fn c2_and_c21_disc_of_another_game_offers_play_other() {
    let mut e = env();
    e.select(e.x, 0);
    let iso = e.y_iso.clone();
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(1);
    let y = e.y;
    assert_eq!(e.app.state(), &State::Rejected { game: e.x, class: MediaClass::OtherGame { other: y } });
    assert!(e.app.drive().has_media(), "rejeitar não ejeta (ADR-0019)");
    let acts = e.app.actions();
    assert_eq!(acts, [(Action::TryOther, true), (Action::PlayOther, false)]);
    e.act(Action::PlayOther, 2);
    assert_eq!(e.launches(), [steam(504230)]);
}

#[test]
fn c3_and_c22_unknown_disc_can_be_adopted_and_then_launches() {
    let mut e = env();
    let stray = e.game_iso("Outro Jogo", Uuid::new_v4());
    e.select(e.x, 0);
    e.app.drive_mut().insert_iso(stray);
    e.app.pump(1);
    assert!(matches!(e.app.state(), State::Rejected { class: MediaClass::Unknown { .. }, .. }));
    assert!(e.app.drive().has_media());
    e.act(Action::Adopt, 2);
    assert_eq!(e.name(), "ADOPT_CONFIRM");
    e.app.dispatch(Intent::Back, 3).unwrap();
    assert_eq!(e.name(), "REJECTED");
    e.act(Action::Adopt, 4);
    e.act(Action::Confirm, 5);
    assert_eq!(e.launches(), [steam(367520)]);
    let g = e.app.catalog.game(e.x).unwrap();
    assert_eq!(g.discs.len(), 2);
    let adopted = g.discs.iter().find(|d| d.origin == DiscOrigin::Adopted).unwrap();
    assert_eq!((adopted.media, adopted.label.as_str()), (DiscMedia::Iso, "OUTRO_JOGO"));
}

#[test]
fn rejection_does_not_eject_until_try_other() {
    let mut e = env();
    e.select(e.x, 0);
    e.app.drive_mut().insert_blank_cdr();
    e.app.pump(1);
    assert_eq!(e.app.state(), &State::Rejected { game: e.x, class: MediaClass::Blank });
    e.act(Action::TryOther, 2);
    assert!(!e.app.drive().has_media());
    e.app.pump(3);
    assert_eq!(e.app.state(), &State::WaitingDisc { game: e.x, tray: TrayHint::Opening });
    assert!(!e.app.media_present());
}

#[test]
fn c4_c5_c6_c7_c10_c11_c19_rejection_classes() {
    type Case = (&'static str, Box<dyn Fn(&mut Env)>, MediaClass);
    let cases: Vec<Case> = vec![
        ("no ini", Box::new(|e| {
            let p = e.dir.path().join("plain.iso");
            fs::write(&p, iso::build_iso("DADOS", &[("README.TXT", b"oi")])).unwrap();
            e.app.drive_mut().insert_iso(p);
        }), MediaClass::NoGameIni),
        ("invalid ini", Box::new(|e| {
            let p = write_iso(&e.dir, "bad", "[disc]\nid = nao-e-uuid\n");
            e.app.drive_mut().insert_iso(p);
        }), MediaClass::InvalidGameIni),
        ("legacy", Box::new(|e| {
            let p = write_iso(&e.dir, "old", "[Game]\nNAME=X\nSTEAMID=620\nPROCESS=p\n");
            e.app.drive_mut().insert_iso(p);
        }), MediaClass::LegacyGameIni),
        ("audio", Box::new(|e| e.app.drive_mut().insert_audio()), MediaClass::Audio),
        ("read error", Box::new(|e| e.app.drive_mut().insert_unreadable()), MediaClass::ReadError),
        ("truncated iso", Box::new(|e| {
            let p = e.dir.path().join("cut.iso");
            let full = fs::read(&e.x_iso).unwrap();
            fs::write(&p, &full[..20 * 2048 + 10]).unwrap();
            e.app.drive_mut().insert_iso(p);
        }), MediaClass::ReadError),
    ];
    for (name, insert, class) in cases {
        let mut e = env();
        e.select(e.x, 0);
        insert(&mut e);
        e.app.pump(1);
        assert_eq!(e.app.state(), &State::Rejected { game: e.x, class }, "{name}");
        assert!(e.launches().is_empty(), "{name}");
    }
}

#[test]
fn c6_extra_keys_on_a_valid_ini_are_ignored_and_the_id_decides() {
    let mut e = env();
    let p = write_iso(&e.dir, "extra", &format!("[disc]\nid = {}\nsteamid = 1\nprocess = calc.exe\n", e.x_disc));
    e.select(e.x, 0);
    e.app.drive_mut().insert_iso(p);
    e.app.pump(1);
    e.app.pump(1 + FICHA_MIN_MS);
    assert_eq!(e.launches(), [steam(367520)]); // do catálogo, não do disco
}

#[test]
fn c13_disc_present_at_boot_is_disarmed_but_selecting_still_launches() {
    let mut e = env_with(|d, x, _| d.insert_iso(x.clone()));
    assert!(e.app.media_present() && !e.app.armed());
    assert_eq!(e.app.state(), &State::Library);
    e.app.pump(1);
    assert!(e.launches().is_empty());
    assert!(e.app.dispatch(Intent::Select(e.y), 2).is_ok()); // y não tem o disco presente
    assert!(matches!(e.app.state(), State::Rejected { class: MediaClass::OtherGame { .. }, .. }));
    e.app.dispatch(Intent::Back, 3).unwrap();
    e.select(e.x, 4);
    e.app.pump(5);
    assert_eq!(e.name(), "IDENTIFIED");
    e.app.pump(5 + FICHA_MIN_MS);
    assert_eq!(e.launches(), [steam(367520)]);
}

#[test]
fn c20_focus_mode_focuses_without_launching_and_launch_mode_launches_then_rearms_on_removal() {
    // focus (padrão)
    let mut e = env();
    let iso = e.y_iso.clone();
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(1);
    assert_eq!(e.app.state(), &State::Library);
    assert!(e.launches().is_empty());
    let y = e.y;
    assert!(e.app.armed());
    assert_eq!(e.app.snapshot().focus_hint, Some(y));

    // launch
    let mut e = env();
    e.app.dispatch(Intent::OpenSettings, 0).unwrap();
    e.app.dispatch(Intent::SetSetting(SettingChange::OnDiscInsert(OnDiscInsert::Launch)), 0).unwrap();
    e.app.dispatch(Intent::Back, 0).unwrap();
    let iso = e.y_iso.clone();
    e.app.drive_mut().insert_iso(iso.clone());
    e.app.pump(1);
    assert_eq!(e.launches(), [steam(504230)]);
    e.app.pump(1 + 3000);
    assert_eq!(e.app.state(), &State::Library);

    // jogo fechou, disco continua: não relança; evento duplicado (C18) também não
    e.app.drive_mut().inject_duplicate_arrival();
    e.app.pump(9000);
    assert_eq!(e.launches().len(), 1);
    assert!(!e.app.armed());

    // tirar e recolocar rearma
    e.app.drive_mut().remove_media();
    e.app.pump(9001);
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(9002);
    assert_eq!(e.launches().len(), 2);
}

#[test]
fn c18_duplicate_arrival_right_after_a_selected_launch_does_not_relaunch() {
    let mut e = env();
    e.app.dispatch(Intent::OpenSettings, 0).unwrap();
    e.app.dispatch(Intent::SetSetting(SettingChange::OnDiscInsert(OnDiscInsert::Launch)), 0).unwrap();
    e.app.dispatch(Intent::Back, 0).unwrap();
    e.select(e.x, 0);
    let iso = e.x_iso.clone();
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(1);
    e.app.pump(1 + FICHA_MIN_MS);
    e.app.pump(1 + FICHA_MIN_MS + 3000);
    e.app.drive_mut().inject_duplicate_arrival();
    e.app.pump(9000);
    assert_eq!(e.launches().len(), 1);
}

#[test]
fn unknown_disc_inserted_without_selection_only_toasts_and_never_adopts() {
    let mut e = env();
    let stray = e.game_iso("Sem Dono", Uuid::new_v4());
    e.app.drive_mut().insert_iso(stray);
    e.app.pump(1);
    assert_eq!(e.app.state(), &State::Library);
    assert_eq!(e.app.snapshot().toast.unwrap().code, "unknown");
    assert!(e.app.actions().is_empty());
}

#[test]
fn media_inserted_outside_the_library_is_never_armed() {
    let mut e = env();
    e.app.dispatch(Intent::OpenSettings, 0).unwrap();
    let iso = e.x_iso.clone();
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(1);
    e.app.dispatch(Intent::Back, 2).unwrap();
    assert!(e.app.media_present() && !e.app.armed());
    assert!(e.launches().is_empty());
}

#[test]
fn c23_removal_while_showing_a_rejection_goes_back_to_waiting() {
    let mut e = env();
    e.select(e.x, 0);
    e.app.drive_mut().insert_audio();
    e.app.pump(1);
    assert_eq!(e.name(), "REJECTED");
    e.app.drive_mut().remove_media();
    e.app.pump(2);
    assert_eq!(e.app.state(), &State::WaitingDisc { game: e.x, tray: TrayHint::Opening });
}

#[test]
fn c15_tray_hints_follow_capabilities_and_failures() {
    let mut e = env();
    e.app.drive_mut().set_capabilities(Capabilities { tray_open: Tri::No, ..Capabilities::ALL });
    e.select(e.x, 0);
    assert_eq!(e.app.state(), &State::WaitingDisc { game: e.x, tray: TrayHint::Manual });
    assert!(e.app.actions().is_empty());

    let mut e = env();
    e.app.drive_mut().set_fail_open_tray(true);
    e.select(e.x, 0);
    assert_eq!(e.app.state(), &State::WaitingDisc { game: e.x, tray: TrayHint::Failed });
    assert_eq!(e.app.actions(), [(Action::RetryTray, false)]);
    e.app.drive_mut().set_fail_open_tray(false);
    e.act(Action::RetryTray, 1);
    assert_eq!(e.app.state(), &State::WaitingDisc { game: e.x, tray: TrayHint::Opening });
}

#[test]
fn launch_error_shows_the_reason_and_confirm_returns() {
    let mut e = env();
    e.app.launcher_mut().fail = Some(LaunchError::SteamMissing);
    e.select(e.x, 0);
    let iso = e.x_iso.clone();
    e.app.drive_mut().insert_iso(iso);
    e.app.pump(1);
    e.app.pump(1 + FICHA_MIN_MS);
    assert!(matches!(e.app.state(), State::LaunchError { reason: LaunchError::SteamMissing, .. }));
    e.act(Action::Confirm, 3);
    assert_eq!(e.app.state(), &State::Library);
}

#[test]
fn c30_selecting_a_game_without_discs_offers_to_burn() {
    let mut e = env();
    let z = e.app.catalog.add_game(Game::new("Sem Disco", GameKind::Steam { app_id: 9 })).unwrap();
    e.select(z, 0);
    assert_eq!(e.app.state(), &State::NoDiscYet { game: z });
    assert_eq!(e.app.actions(), [(Action::Burn, true)]);
    e.app.dispatch(Intent::Back, 1).unwrap();
    e.select(z, 2);
    e.act(Action::Burn, 3);
    assert_eq!(e.app.state(), &State::RegInsert { game: Some(z), tray: TrayHint::Opening });
}

// ---------- drive ----------

#[test]
fn c24_drive_removed_in_each_kind_of_state() {
    // biblioteca: só avisa
    let mut e = env();
    e.app.drive_mut().disconnect();
    e.app.pump(1);
    assert_eq!(e.app.state(), &State::Library);
    assert_eq!(e.app.snapshot().toast.unwrap().code, "drive_removed");
    assert_eq!(e.app.snapshot().drive, "removed");
    // selecionar sem drive
    e.select(e.x, 2);
    assert_eq!(e.app.state(), &State::DriveProblem { reason: ProblemReason::Removed });
    // reconectar volta
    e.app.drive_mut().reconnect();
    e.app.pump(3);
    assert_eq!(e.app.state(), &State::Library);

    // esperando disco -> problema de drive
    let mut e = env();
    e.select(e.x, 0);
    e.app.drive_mut().disconnect();
    e.app.pump(1);
    assert_eq!(e.app.state(), &State::DriveProblem { reason: ProblemReason::Removed });
    assert_eq!(e.app.actions(), [(Action::ChooseDrive, true)]);
    e.act(Action::ChooseDrive, 2);
    assert_eq!(e.name(), "SETTINGS");
}

#[test]
fn no_drive_at_all_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let mut d = FakeIsoDrive::new(dir.path());
    d.disconnect();
    let mut app = App::boot(Ok(Catalog::default()), None, d, Rec::default(), 0);
    assert_eq!(app.dispatch(Intent::AddGame, 1), Ok(()));
    assert_eq!(app.state(), &State::DriveProblem { reason: ProblemReason::NoDrive });
}

// ---------- cadastro e gravação ----------

fn add_steam_game(e: &mut Env, name: &str, app_id: u32, t: u64) {
    e.app.dispatch(Intent::CreateGame(NewGame { name: name.into(), kind: GameKind::Steam { app_id } }), t).unwrap();
}

#[test]
fn c25_a_drive_that_cannot_burn_blocks_registration_up_front() {
    let mut e = env();
    e.app.drive_mut().set_capabilities(Capabilities { write_cdr: Tri::No, write_cdrw: Tri::No, ..Capabilities::ALL });
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    assert_eq!(e.app.state(), &State::DriveProblem { reason: ProblemReason::CannotBurn });
}

#[test]
fn full_registration_on_cdrw_then_play_the_burned_disc() {
    let mut e = env();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    assert_eq!(e.name(), "REG_INSERT");
    e.app.drive_mut().insert_blank_cdrw();
    e.app.pump(1);
    assert_eq!(e.name(), "REG_CHOOSE_GAME");
    add_steam_game(&mut e, "Portal 2", 620, 2);
    let State::RegLabelPreview { game, label } = e.app.state().clone() else { panic!("{:?}", e.app.state()) };
    assert_eq!(label, "PORTAL_2");
    e.app.dispatch(Intent::LabelEdit("Portal Dois!".into()), 3).unwrap();
    assert!(matches!(e.app.state(), State::RegLabelPreview { label, .. } if label == "PORTAL_DOIS"));
    e.act(Action::Continue, 4); // CD-RW: sem aviso de CD-R
    assert_eq!(e.app.state(), &State::BurnDone { game });

    let g = e.app.catalog.game(game).unwrap();
    assert_eq!(g.discs.len(), 1);
    assert_eq!((g.discs[0].media, g.discs[0].origin, g.discs[0].label.as_str()), (DiscMedia::Iso, DiscOrigin::Burned, "PORTAL_DOIS"));
    // o .iso gerado tem o GAME.INI com o disc_id do catálogo e não vaza campos de execução
    let burned = e.app.drive().burned_path().unwrap();
    let (label, ini) = iso::read_label_and_file(&mut fs::File::open(burned).unwrap(), "GAME.INI", 5000).unwrap();
    let ini_text = String::from_utf8(ini.unwrap()).unwrap();
    assert_eq!(label, "PORTAL_DOIS");
    assert!(ini_text.contains(&g.discs[0].disc_id.to_string()) && !ini_text.contains("620") && !ini_text.contains("steam"));

    e.act(Action::Done, 5);
    assert_eq!(e.app.state(), &State::Library);

    // reinserir o mesmo disco gravado e jogar
    e.app.drive_mut().remove_media();
    e.app.pump(6);
    let burned = e.app.drive().out_dir().join("PORTAL_DOIS-1.iso");
    e.app.drive_mut().insert_iso(burned);
    e.app.pump(7);
    e.select(game, 8);
    e.app.pump(9);
    e.app.pump(9 + FICHA_MIN_MS);
    assert_eq!(e.launches(), [steam(620)]);
}

#[test]
fn cdr_requires_the_single_write_warning() {
    let mut e = env();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_blank_cdr();
    e.app.pump(1);
    add_steam_game(&mut e, "Hades", 1145360, 2);
    e.act(Action::Continue, 3);
    assert_eq!(e.name(), "REG_CDR_WARNING");
    e.app.dispatch(Intent::Back, 4).unwrap();
    assert_eq!(e.name(), "REG_LABEL_PREVIEW");
    e.act(Action::Continue, 5);
    e.act(Action::Burn, 6);
    assert_eq!(e.name(), "BURN_DONE");
    assert_eq!(e.app.snapshot().progress, Some(100));
}

#[test]
fn c26_used_cdr_and_other_unwritable_media_are_rejected_and_kept_until_try_other() {
    for (setup, reason) in [
        (0, RegRejectReason::CdrUsed),
        (1, RegRejectReason::Audio),
        (2, RegRejectReason::ReadError),
        (3, RegRejectReason::NotWritable),
    ] {
        let mut e = env();
        e.app.dispatch(Intent::AddGame, 0).unwrap();
        let iso = e.y_iso.clone();
        match setup {
            0 => e.app.drive_mut().insert_cdr_iso(iso),
            1 => e.app.drive_mut().insert_audio(),
            2 => e.app.drive_mut().insert_unreadable(),
            _ => e.app.drive_mut().insert_iso(iso), // CD-ROM prensado
        }
        e.app.pump(1);
        assert_eq!(e.app.state(), &State::RegRejected { game: None, reason }, "{setup}");
        assert!(e.app.drive().has_media());
        e.act(Action::TryOther, 2);
        assert!(!e.app.drive().has_media());
        assert_eq!(e.name(), "REG_INSERT");
    }
}

#[test]
fn erase_needs_double_confirmation_and_drops_the_old_disc_only_after_success() {
    // CD-RW com o disco de X gravado
    let mut e = env();
    let iso = e.x_iso.clone();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_rewritable_iso(iso);
    e.app.pump(1);
    assert_eq!(e.app.state(), &State::RegEraseConfirm { game: None, step: 1, old_disc: Some(e.x_disc) });
    assert_eq!(e.app.dispatch(Intent::HoldComplete, 2), Err(IntentError::Ignored), "sem o 1º passo não apaga");
    e.act(Action::Confirm, 3);
    assert_eq!(e.app.actions(), []);
    // apagamento falha: a associação continua
    e.app.drive_mut().set_fail_erase(true);
    e.app.dispatch(Intent::HoldComplete, 4).unwrap();
    assert!(matches!(e.app.state(), State::BurnFailed { reason: BurnFailure::EraseError, cdrw: true, .. })); // C29
    assert!(e.app.catalog.game_of_disc(e.x_disc).is_some());
    // tenta de novo e funciona
    e.app.drive_mut().set_fail_erase(false);
    e.act(Action::EraseRetry, 5);
    assert_eq!(e.name(), "REG_CHOOSE_GAME");
    assert!(e.app.catalog.game_of_disc(e.x_disc).is_some(), "EraseRetry não conhece o disc_id antigo");
}

#[test]
fn erase_success_removes_the_old_disc_key() {
    let mut e = env();
    let iso = e.x_iso.clone();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_rewritable_iso(iso);
    e.app.pump(1);
    e.act(Action::Confirm, 2);
    e.app.dispatch(Intent::HoldComplete, 3).unwrap();
    assert_eq!(e.name(), "REG_CHOOSE_GAME");
    assert!(e.app.catalog.game_of_disc(e.x_disc).is_none());
    assert!(e.app.catalog.game(e.x).unwrap().discs.is_empty());
}

#[test]
fn erase_back_ejects() {
    let mut e = env();
    let iso = e.x_iso.clone();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_rewritable_iso(iso);
    e.app.pump(1);
    e.app.dispatch(Intent::Back, 2).unwrap();
    assert_eq!(e.name(), "REG_INSERT");
    assert!(!e.app.drive().has_media());
}

#[test]
fn c16_failed_burn_loses_a_cdr_and_a_cdrw_recovers_via_erase_retry() {
    // CD-R: perdido, nada entra no catálogo
    let mut e = env();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_blank_cdr();
    e.app.pump(1);
    add_steam_game(&mut e, "Hades", 1, 2);
    e.app.drive_mut().set_fail_burn(true);
    e.act(Action::Continue, 3);
    e.act(Action::Burn, 4);
    assert!(matches!(e.app.state(), State::BurnFailed { reason: BurnFailure::WriteError, cdrw: false, .. }));
    assert_eq!(e.app.actions(), [(Action::TryOther, true)]);
    assert_eq!(e.app.catalog.games.iter().map(|g| g.discs.len()).sum::<usize>(), 2, "só os 2 discos originais");

    // CD-RW: apaga e tenta de novo, mantendo jogo e rótulo
    let mut e = env();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_blank_cdrw();
    e.app.pump(1);
    add_steam_game(&mut e, "Hades", 1, 2);
    e.app.drive_mut().set_fail_burn(true);
    e.act(Action::Continue, 3);
    assert_eq!(e.app.actions(), [(Action::EraseRetry, true), (Action::TryOther, false)]);
    e.app.drive_mut().set_fail_burn(false);
    e.act(Action::EraseRetry, 4);
    assert!(matches!(e.app.state(), State::RegLabelPreview { label, .. } if label == "HADES"));
    e.act(Action::Continue, 5);
    assert_eq!(e.name(), "BURN_DONE");
}

#[test]
fn registration_from_game_options_and_the_burn_another_flow() {
    let mut e = env();
    e.app.dispatch(Intent::Options(e.x), 0).unwrap();
    e.act(Action::BurnAnother, 1);
    e.app.drive_mut().insert_blank_cdrw();
    e.app.pump(2);
    assert!(matches!(e.app.state(), State::RegLabelPreview { game, label } if *game == e.x && label == "HOLLOW_KNIGHT"));
    e.act(Action::Continue, 3);
    assert_eq!(e.app.catalog.game(e.x).unwrap().discs.len(), 2); // 1 jogo : N discos
}

#[test]
fn media_already_present_when_starting_registration_is_read_immediately() {
    let mut e = env_with(|d, _, _| d.insert_blank_cdrw());
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    assert_eq!(e.name(), "REG_CHOOSE_GAME");
}

// ---------- gestão ----------

#[test]
fn removing_a_game_orphans_its_discs_and_unlinking_works() {
    let mut e = env();
    e.app.dispatch(Intent::Options(e.x), 0).unwrap();
    e.act(Action::RemoveGame, 1);
    e.app.dispatch(Intent::Back, 2).unwrap();
    assert_eq!(e.name(), "GAME_OPTIONS");
    e.act(Action::UnlinkDisc(e.x_disc), 3);
    assert!(e.app.catalog.game(e.x).unwrap().discs.is_empty());
    assert_eq!(e.app.dispatch(Intent::Action(Action::UnlinkDisc(Uuid::new_v4())), 4), Err(IntentError::Ignored));
    e.app.dispatch(Intent::RenameGame(e.x, "  HK  ".into()), 5).unwrap();
    assert_eq!(e.app.catalog.game(e.x).unwrap().name, "HK");
    e.act(Action::RemoveGame, 6);
    e.act(Action::Confirm, 7);
    assert!(e.app.catalog.game(e.x).is_none());
    assert_eq!(e.app.state(), &State::Library);
}

#[test]
fn intents_invalid_for_the_state_are_ignored() {
    let mut e = env();
    for i in [Intent::Back, Intent::HoldComplete, Intent::ChooseGame(e.x), Intent::LabelEdit("x".into()), Intent::Action(Action::Done), Intent::SetSetting(SettingChange::LoadingMinMs(1)), Intent::Select(Uuid::new_v4())] {
        assert_eq!(e.app.dispatch(i, 0), Err(IntentError::Ignored));
    }
    e.select(e.x, 0);
    assert_eq!(e.app.dispatch(Intent::Select(e.y), 1), Err(IntentError::Ignored));
    assert_eq!(e.app.dispatch(Intent::AddGame, 1), Err(IntentError::Ignored));
}

#[test]
fn invalid_new_games_are_refused_by_the_catalog() {
    let mut e = env();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_blank_cdrw();
    e.app.pump(1);
    let bad = Intent::CreateGame(NewGame { name: "x".into(), kind: GameKind::Steam { app_id: 0 } });
    assert!(matches!(e.app.dispatch(bad, 2), Err(IntentError::Invalid(_))));
    assert_eq!(e.name(), "REG_CHOOSE_GAME");
}

// ---------- catálogo ----------

#[test]
fn c27_corrupt_catalog_is_preserved_and_can_be_restored_from_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.json");
    let mut c = Catalog::default();
    c.add_game(Game::new("A", GameKind::Steam { app_id: 1 })).unwrap();
    c.save(&path).unwrap();
    c.add_game(Game::new("B", GameKind::Steam { app_id: 2 })).unwrap();
    c.save(&path).unwrap();
    c.add_game(Game::new("C", GameKind::Steam { app_id: 3 })).unwrap();
    c.save(&path).unwrap(); // 3 gravações = 2 backups (A+B e A)
    fs::write(&path, "{quebrado").unwrap();

    let boot = |dir: &TempDir| {
        App::boot(Catalog::load(&path), Some(path.clone()), FakeIsoDrive::new(dir.path()), Rec::default(), 0)
    };
    let mut app = boot(&dir);
    assert_eq!(app.state(), &State::CatalogError { backups: 2 });
    assert_eq!(fs::read_to_string(&path).unwrap(), "{quebrado", "não sobrescreve");
    assert_eq!(app.actions(), [(Action::RestoreBackup(0), true), (Action::RestoreBackup(1), false), (Action::StartEmpty, false)]);
    assert_eq!(app.dispatch(Intent::Back, 0), Err(IntentError::Ignored));
    app.dispatch(Intent::Action(Action::RestoreBackup(0)), 1).unwrap();
    assert_eq!(app.state(), &State::Library);
    assert_eq!(app.catalog.games.len(), 2); // backup mais novo: A+B
    assert_eq!(fs::read_to_string(dir.path().join("catalog.json.corrupt-1")).unwrap(), "{quebrado");

    fs::write(&path, "lixo").unwrap();
    let mut app = boot(&dir);
    app.dispatch(Intent::Action(Action::StartEmpty), 2).unwrap();
    assert!(app.catalog.games.is_empty() && app.state() == &State::Library);
    assert!(dir.path().join("catalog.json.corrupt-2").exists());
}

#[test]
fn changes_are_persisted_to_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("c.json");
    let mut app = App::boot(Catalog::load(&path), Some(path.clone()), FakeIsoDrive::new(dir.path()), Rec::default(), 0);
    app.dispatch(Intent::OpenSettings, 0).unwrap();
    app.dispatch(Intent::SetSetting(SettingChange::OnDiscInsert(OnDiscInsert::Launch)), 0).unwrap();
    assert!(app.persist_error.is_none());
    assert_eq!(Catalog::load(&path).unwrap().settings.on_disc_insert, OnDiscInsert::Launch);
}

#[test]
fn update_game_edits_kind_fields_with_the_creation_rules() {
    use insert_disc_core::catalog::GameKind;
    let mut e = env();
    let abs = |p: &str| if cfg!(windows) { std::path::PathBuf::from(format!("C:/{p}")) } else { std::path::PathBuf::from(format!("/{p}")) };
    let custom = |exe: &str, args: &[&str]| GameKind::Custom { executable: abs(exe), args: args.iter().map(|s| s.to_string()).collect(), working_dir: None, requires_elevation: true };
    // só em GAME_OPTIONS
    let upd = |name: &str, kind| Intent::UpdateGame(e.x, NewGame { name: name.into(), kind });
    assert_eq!(e.app.dispatch(upd("Novo", custom("g.exe", &[])), 0), Err(IntentError::Ignored));

    e.app.dispatch(Intent::Options(e.x), 1).unwrap();
    e.app.dispatch(upd("  Editado ", custom("games/g.exe", &["--fullscreen"])), 2).unwrap();
    let g = e.app.catalog.game(e.x).unwrap();
    assert_eq!(g.name, "Editado");
    assert_eq!(g.kind, custom("games/g.exe", &["--fullscreen"]));
    assert_eq!(g.discs.len(), 1, "discos e capa são preservados");

    // mesma validação da criação: relativo, .bat com argumentos e nome vazio são recusados sem alterar nada
    let before = e.app.catalog.clone();
    for bad in [upd("X", GameKind::Custom { executable: "rel.exe".into(), args: vec![], working_dir: None, requires_elevation: false }), upd("X", custom("run.bat", &["a"])), upd(" ", custom("g.exe", &[]))] {
        assert!(matches!(e.app.dispatch(bad, 3), Err(IntentError::Invalid(_))));
    }
    assert_eq!(e.app.catalog, before);
}

// ---------- A5: operações assíncronas do drive ----------

#[test]
fn a5_slow_drive_shows_reading_then_burning_with_progress_then_verifying() {
    let mut e = env();
    e.app.drive_mut().set_op_delay_ms(1000);

    // leitura de identificação: READING dura até o resultado chegar
    e.select(e.x, 0);
    e.app.drive_mut().insert_iso(e.x_iso.clone());
    e.app.pump(10);
    assert_eq!(e.name(), "READING");
    e.app.pump(500);
    assert_eq!(e.name(), "READING");
    e.app.pump(1100);
    assert_eq!(e.name(), "IDENTIFIED");
    e.app.pump(1800);
    assert_eq!(e.name(), "LAUNCHING");
    e.app.pump(6000); // fim do loading mínimo

    // cadastro: leitura do virgem (REG_READING) -> gravação com progresso -> verificação -> pronto
    e.app.dispatch(Intent::AddGame, 6100).unwrap();
    e.app.drive_mut().remove_media();
    e.app.drive_mut().insert_blank_cdrw();
    e.app.pump(6101);
    assert_eq!(e.name(), "REG_READING");
    e.app.pump(7200);
    assert_eq!(e.name(), "REG_CHOOSE_GAME");
    e.app.dispatch(Intent::CreateGame(NewGame { name: "Portal 2".into(), kind: GameKind::Steam { app_id: 620 } }), 7300).unwrap();
    e.act(Action::Continue, 7400);
    assert_eq!(e.name(), "BURNING");
    assert_eq!(e.app.snapshot().progress, Some(0));
    e.app.pump(7900);
    assert_eq!(e.name(), "BURNING");
    assert_eq!(e.app.snapshot().progress, Some(50));
    e.app.pump(8500); // gravação termina e a releitura (VERIFYING) começa
    assert_eq!(e.name(), "VERIFYING");
    e.app.pump(9600);
    assert_eq!(e.name(), "BURN_DONE");
}

#[test]
fn a5_removing_the_disc_or_going_back_during_a_read_cancels_it() {
    let mut e = env();
    e.app.drive_mut().set_op_delay_ms(1000);
    e.select(e.x, 0);
    e.app.drive_mut().insert_iso(e.x_iso.clone());
    e.app.pump(10);
    assert_eq!(e.name(), "READING");
    e.app.drive_mut().remove_media();
    e.app.pump(20);
    assert_eq!(e.name(), "WAITING_DISC");
    e.app.pump(2000);
    assert_eq!(e.name(), "WAITING_DISC", "resultado tardio é descartado");

    e.app.drive_mut().insert_iso(e.x_iso.clone());
    e.app.pump(2100);
    assert_eq!(e.name(), "READING");
    e.app.dispatch(Intent::Back, 2200).unwrap();
    e.app.pump(4000);
    assert_eq!(e.name(), "LIBRARY");
    assert!(e.launches().is_empty());
}

#[test]
fn a5_pulling_the_disc_during_a_burn_fails_it_as_drive_removed() {
    let mut e = env();
    e.app.dispatch(Intent::AddGame, 0).unwrap();
    e.app.drive_mut().insert_blank_cdr();
    e.app.pump(1);
    add_steam_game(&mut e, "Hades", 1, 2);
    e.app.drive_mut().set_op_delay_ms(1000);
    e.act(Action::Continue, 3);
    e.act(Action::Burn, 4); // aviso de CD-R
    assert_eq!(e.name(), "BURNING");
    e.app.drive_mut().remove_media();
    e.app.pump(1500);
    assert!(matches!(e.app.state(), State::BurnFailed { reason: BurnFailure::DriveRemoved, .. }), "{:?}", e.app.state());
}

// ---------- C2: escolha do drive ----------

#[test]
fn c2_a_chosen_drive_that_disappears_is_a_drive_problem_and_auto_follows_what_exists() {
    let mut e = env();
    let snap = e.app.snapshot();
    assert_eq!(snap.drives.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["fake:0"]);
    assert_eq!(snap.drive_in_use.as_deref(), Some("fake:0"));

    // escolha explícita de uma unidade que existe: mantém e persiste nas configurações
    e.app.dispatch(Intent::OpenSettings, 0).unwrap();
    e.app.dispatch(Intent::SetSetting(SettingChange::Drive(Some("fake:0".into()))), 1).unwrap();
    assert_eq!(e.app.catalog.settings.drive.as_deref(), Some("fake:0"));
    e.app.dispatch(Intent::Back, 2).unwrap();

    // some: sem fallback silencioso
    e.app.drive_mut().disconnect();
    e.app.pump(3);
    e.select(e.x, 4);
    assert_eq!(e.name(), "DRIVE_PROBLEM");
    let snap = e.app.snapshot();
    assert!(snap.drives.is_empty() && snap.drive_in_use.is_none());

    // volta o drive e, em "auto", segue a primeira unidade existente
    e.app.dispatch(Intent::Back, 5).unwrap();
    e.app.drive_mut().reconnect();
    e.app.pump(6);
    e.app.dispatch(Intent::OpenSettings, 7).unwrap();
    e.app.dispatch(Intent::SetSetting(SettingChange::Drive(None)), 8).unwrap();
    assert_eq!(e.app.snapshot().drive_in_use.as_deref(), Some("fake:0"));
}

// ---------- C12: leitura sem resposta ----------

#[test]
fn c12_a_read_that_never_answers_becomes_a_read_error_after_the_timeout() {
    let mut e = env();
    e.app.drive_mut().set_op_delay_ms(600_000); // o drive "trava"
    e.select(e.x, 0);
    e.app.drive_mut().insert_iso(e.x_iso.clone());
    e.app.pump(10);
    assert_eq!(e.name(), "READING");
    e.app.pump(READ_TIMEOUT_MS - 1);
    assert_eq!(e.name(), "READING");
    e.app.pump(10 + READ_TIMEOUT_MS);
    assert!(matches!(e.app.state(), State::Rejected { class: MediaClass::ReadError, .. }), "{:?}", e.app.state());
    // o resultado tardio do drive não muda nada
    e.app.drive_mut().set_op_delay_ms(0);
    e.app.pump(20_000);
    assert!(matches!(e.app.state(), State::Rejected { class: MediaClass::ReadError, .. }));
}
