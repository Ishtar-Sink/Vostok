<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

Mede, num protótipo descartável fora da linha principal, quanto o painel pesa no binário com cada opção de servidor e de empacotamento da interface, e propõe o limite para a CLI continuar leve.
Perfil mecanica, mas com builds longos (cerca de 5 compilações de release). Risco: o empacotamento em React passar do limite, o que favorece a variante com Preact.
Roda em paralelo com o levantamento (TASK-01) e desbloqueia o protótipo funcional (TASK-03).

## O que foi feito

- Foi montado um protótipo descartável, fora da linha principal do projeto, para medir quanto o painel aumenta o programa.
- Foram comparadas 5 versões: o programa atual, sem painel, e quatro com painel, combinando duas formas de servidor e duas bibliotecas de interface (React e Preact).
- A versão mais leve, com Preact, aumenta o programa em cerca de 0,5 MB (11%). A versão com React aumenta cerca de 0,7 MB e fica logo acima do limite proposto.
- Proposta de limite: o painel pode aumentar o programa até 5,3 MB instalado e 2,0 MB compactado. A versão com Preact cabe com pouca folga. As medições não mostram se a interface se comporta igual nessa biblioteca, o que precisa ser testado antes de adotar.
- O painel exige Node.js para ser compilado, o que muda a instalação e a publicação das versões. O estudo descreve duas saídas para isso.
