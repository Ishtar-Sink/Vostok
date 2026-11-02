<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Cria a detecção das cores que o repositório já usa, para que a skill proponha a identidade em vez de perguntar do zero. Ela ignora pastas geradas e arquivos de segredo. Perfil padrão, sem risco alto; depende do comando da TASK-01 e libera a skill (TASK-04).

## O que foi feito

O harness passou a encontrar sozinho as cores que o repositório já usa nas folhas de estilo e no tema do Tailwind, listando primeiro as que parecem de marca, para que a customização dos documentos comece de uma proposta em vez de uma pergunta em branco. A busca nunca entra em pastas geradas nem em arquivos de segredo, mostra no máximo dez cores com o arquivo e a linha de cada uma e diz claramente quando não encontra nenhuma.
