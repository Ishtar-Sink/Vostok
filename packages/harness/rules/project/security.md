# Segurança

Define como o agente e o código protegem dados sensíveis.

[project/security-01]: O agente não lê `.env`, chaves, certificados nem arquivos de credenciais, salvo pedido explícito do usuário.
[project/security-02]: Nunca exiba segredos em respostas, logs, evidências ou documentos; mascare-os se aparecerem.
[project/security-03]: Credenciais e URLs privadas só por variáveis de ambiente, nunca no código nem em arquivos versionados.
[project/security-04]: Nenhum arquivo de segredo é commitado: `.env` fica no `.gitignore`, e um `.env.example` versionado lista as variáveis sem valores.
[project/security-05]: Consultas ao banco nunca são montadas por concatenação de entrada; a entrada é validada na fronteira da API.
[project/security-06]: Ao encontrar dado sensível (pessoal ou credencial) por acaso, não o copie para outro lugar e avise o usuário.
