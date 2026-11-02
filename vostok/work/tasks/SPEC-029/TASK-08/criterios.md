### Concluída quando

- [x] As telas specs, spec, board mostram os dados reais da pasta com os estados de carregamento, vazio e erro (RF-09, CT-03).

  > Como comprovar: em `task-08-testes.txt`, ver os testes passando; nos prints, ver as telas com dados reais.

- [x] Nenhuma tela importa outra feature, e o build do frontend passa (RF-03).

  > Como comprovar: `grep -rn "from '../" apps/painel/src/features/specs apps/painel/src/features/spec apps/painel/src/features/board` não cita outra feature, e o build em `task-08-testes.txt` passa.
