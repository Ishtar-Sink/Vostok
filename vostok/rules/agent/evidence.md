# Evidências

Define como o agente escreve o sinal de falha das evidências.

[agent/evidence-01]: Cada texto entre crases da `Falha:` é algo que só aparece no log quando a tarefa falhou, como `FAILED` ou `rm_na_referencia=1`.
[agent/evidence-02]: Nunca cite na `Falha:` um texto que o `Trecho:` exige, nem parte dele: com o `Trecho:` `mp4_referencia=152`, a `Falha:` `mp4_referencia` sempre aparece. A ausência de um texto do `Trecho:` já é falha do `Trecho:`.
[agent/evidence-03]: Condição sobre o valor ("diferente de", "maior que", "menor que", "outra versão") ou ausência ("sem") fica fora das crases. Entre crases vai o valor errado por extenso: "se aparecer `rm_na_referencia=1`", e não "se `rm_na_referencia` for maior que 0".
[agent/evidence-04]: Antes de pedir a conferência, corrija a `Falha:` que o `vostok plan approve validate`, o `vostok task adjust validate` ou o `vostok task evidence --antecipada` apontarem.
