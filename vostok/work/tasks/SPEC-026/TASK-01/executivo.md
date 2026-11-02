<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Cria o comando que grava a identidade visual de um repositório (cor principal, cor secundária, nome e ícone) e calcula a partir dela as cores dos temas claro e escuro dos documentos finais, mantendo o texto legível. Também permite voltar à identidade padrão. Perfil padrão, sem risco alto; libera a detecção de cores (TASK-02) e a skill (TASK-04).

## O que foi feito

O harness ganhou o comando que grava a identidade visual do repositório. Com uma cor principal, e opcionalmente uma secundária, ele calcula as cores dos temas claro e escuro dos documentos finais, mantendo o texto legível: quando uma cor clara demais (um amarelo, por exemplo) não daria leitura, o comando a escurece até passar no contraste mínimo e avisa a troca. Cada cor alterada aparece com o valor anterior e o novo, as cores de sucesso, aviso e erro não mudam, e um único pedido volta tudo à identidade padrão. Entradas inválidas são recusadas sem alterar nada. O nome e o ícone também ficam gravados, prontos para o topo dos documentos.
