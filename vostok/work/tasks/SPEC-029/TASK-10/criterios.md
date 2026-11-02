### Concluída quando

- [x] As telas settings mostram os dados reais da pasta com os estados de carregamento, vazio e erro (RF-10, CT-11).

  > Como comprovar: em `task-10-testes.txt`, ver os testes passando; nos prints, ver as telas com dados reais.

- [x] Nenhuma tela importa outra feature, e o build do frontend passa (RF-03).

  > Como comprovar: `grep -rn "from '../" apps/painel/src/features/settings` não cita outra feature, e o build em `task-10-testes.txt` passa.
