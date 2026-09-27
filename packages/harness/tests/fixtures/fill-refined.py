#!/usr/bin/env python3
"""Preenche uma spec refinada de teste. Uso: fill-refined.py <arquivo> [VAR=valor ...]
Valores padrão são válidos; os argumentos sobrescrevem variáveis específicas."""
import re
import sys

DEFAULTS = {
    'SPEC_SUMMARY': 'Resumo.',
    'OBJECTIVE': 'Objetivo.',
    'CURRENT_STATE': 'Estado atual.',
    'DEFECT_CURRENT': 'a', 'DEFECT_EXPECTED': 'b', 'DEFECT_REPRODUCTION': 'c', 'DEFECT_ROOT_CAUSE': 'd',
    'SCOPE_IN': '- Dentro.', 'SCOPE_OUT': '- Fora.',
    'FUNCTIONAL_REQUIREMENTS': '| RF-01 | O sistema deve A. |\n| RF-02 | O sistema deve B. |',
    'NON_FUNCTIONAL_REQUIREMENTS': '| RNF-01 | Rodar em bash 3.2. | Suíte no macOS. |',
    'BUSINESS_RULES': '| - | - |',
    'MAIN_FLOW': '1. Passo.', 'ALTERNATIVE_FLOWS': '- Nenhum.',
    'EDGE_CASES': '| CB-01 | Situação. | Comportamento. | RF-01 |',
    'CONTRACTS': '```bash\ncmd <arg>\n```',
    'AFFECTED_COMPONENTS': '| `AGENTS.md` | alterar | Motivo. |\n| `nao/existe/novo.sh` | criar | Motivo. |',
    'ASSUMPTIONS': '- Premissa.',
    'RISKS': '| Risco. | baixo | Mitigação. |',
    'RESEARCH_QUESTIONS': '| PQ-01 | Pergunta? | Fonte. |',
    'THESIS': 'Tese.', 'THESIS_ARGUMENTS': '| T-01 | Arg. | Evid. |\n| T-02 | Arg. | Evid. |',
    'ANTITHESIS': 'Antítese.', 'ANTITHESIS_ARGUMENTS': '| A-01 | Contra. | Evid. | T-01 |',
    'SYNTHESIS': 'Síntese.', 'SYNTHESIS_CONCLUSIONS': '| S-01 | Conclusão. | T-01, T-02, A-01 |',
    'TEST_CASES': '| CT-01 | Cenário. | Passos. | Resultado. | RF-01, RF-02 |',
    'ACCEPTANCE_CRITERIA': '- [ ] CA-01: Critério (RF-01, RF-02, S-01, PQ-01). Cita `#{VAR}` em crases.',
    'REFERENCES': '| arquivo | `AGENTS.md` | Uso. |',
    'WHAT_CHANGED': '| 1 | Assunto | Antes | Depois |',
    'DECISIONS': '| 1 | Decisão | Alternativa | inferência A |',
    'PENDING': '| - | Nenhuma | - |',
}

path = sys.argv[1]
values = dict(DEFAULTS)
for arg in sys.argv[2:]:
    key, value = arg.split('=', 1)
    values[key] = value.replace('\\n', '\n')

text = open(path, encoding='utf-8').read()
text = re.sub(r'#\{([A-Z0-9_]+)\}', lambda m: values.get(m.group(1), m.group(0)), text)
open(path, 'w', encoding='utf-8').write(text)
