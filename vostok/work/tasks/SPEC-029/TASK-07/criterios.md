### Concluída quando

- [x] As telas dashboard, inbox mostram os dados reais da pasta com os estados de carregamento, vazio e erro (RF-06, RF-11, CT-03).

  > Como comprovar: em `task-07-testes.txt`, ver os testes passando; nos prints, ver as telas com dados reais.

- [x] Nenhuma tela importa outra feature, e o build do frontend passa (RF-03).

  > Como comprovar: `grep -rn "from '../" apps/painel/src/features/dashboard apps/painel/src/features/inbox` não cita outra feature, e o build em `task-07-testes.txt` passa.
