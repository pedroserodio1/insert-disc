# ADR-0015: i18n pt-BR/en; capas online opcionais

**Status:** aceito (Etapa 1)

## Decisão
- **i18n desde o início:** textos da UI fora do código, com pt-BR e en na fase 1. A documentação do projeto fica em pt-BR.
- **Capas:** locais por padrão (cache da Steam, imagem do usuário, placeholder com o nome). Um serviço online é **opcional e desligado por padrão**; o usuário informa a própria chave de API. As capas são baixadas uma vez e guardadas localmente.

## Consequências
- Nenhuma chave de API embutida no binário.
- Termos e limites do serviço: [W10](../RISKS-AND-SPIKES.md#w10-capas-online). Armazenamento da chave: [Q8](../OPEN-QUESTIONS.md#q8-onde-guardar-a-chave-de-api).
