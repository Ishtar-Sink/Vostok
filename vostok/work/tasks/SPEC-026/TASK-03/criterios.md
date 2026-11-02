### Concluída quando

- [x] O nome configurado aparece no topo dos três documentos, escapado, e o rodapé cita o Vostok (RF-06, RN-04, CB-08).

  > Como comprovar: o teste `brand_name_replaces_vostok_in_the_header` passa e `executivo-marca.png` mostra "Atria" no topo.

- [x] O ícone configurado vai embutido no topo, sem requisição externa (RF-07, RNF-02).

  > Como comprovar: o teste `brand_icon_is_embedded_in_the_header` passa e o print mostra o círculo verde.

- [x] Ícone ausente, grande demais ou de formato não suportado faz o build falhar com o motivo (CB-02, CB-03, RNF-05).

  > Como comprovar: os testes `brand_missing_icon_fails_the_build` e `brand_large_or_unknown_icon_fails_the_build` passam.

- [x] Sem identidade configurada, os documentos ficam como hoje e os goldens não mudam (RF-08, RNF-04).

  > Como comprovar: o teste `brand_empty_keeps_the_vostok_identity` e o `golden_finalize` passam, sem arquivo de fixture alterado.

- [x] Os IDs do mapa de cobertura ficam numa linha só (RF-12).

  > Como comprovar: o teste `coverage_ids_do_not_wrap` passa e `cobertura-ids.png` mostra `RNF-01` inteiro.

- [x] Nenhuma cor fixa nova nos templates (RNF-01).

  > Como comprovar: o teste `visual_colors_come_only_from_documents_yml` passa.
