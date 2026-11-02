### Concluída quando

- [x] O app do painel compila e tem o roteador, as pendências e os componentes testados (RF-03, CT-03).

  > Como comprovar: em `frontend-base.txt`, ver os testes passando sem falhas.

- [x] A base aplica a marca da pasta com o menu lateral e o topo, nos tokens do Atria (RF-04, RNF-05).

  > Como comprovar: abrir `base-claro.png` e ver a marca no menu, as specs em andamento e o topo.

- [x] Cada feature tem uma página provisória ligada ao roteador, e nenhuma feature importa outra (RF-03).

  > Como comprovar: `pnpm --dir apps/painel build` passa, e `grep -rn "from '../" apps/painel/src/features` não mostra outra feature.
