#!/usr/bin/env python3
"""Mede a legibilidade de textos em Markdown (RNF-04, CT-16 da SPEC-055).

Uso:
  medir.py ARQUIVO                 imprime as medidas de um texto
  medir.py ANTES DEPOIS            compara os dois e sai com 1 se algum limite falhar
"""
import re
import sys

ID = re.compile(r"\b(?:RF|RNF|RN|CT|CB|CA|CU|DEC|DT|EV)-\d+\b")
DEFINED_ID = re.compile(r"^\|\s*((?:RF|RNF|RN|CT|CB)-\d+)\s*\|", re.M)
FRONT_MATTER = re.compile(r"\A---\n.*?\n---\n", re.S)
FENCE = re.compile(r"^```.*?^```", re.S | re.M)
SENTENCE_END = re.compile(r"(?<=[.!?;:])\s+")
LIST_MARK = re.compile(r"^(?:[-*]\s+|\d+\.\s+|>\s*)+")

MIN_DROP_ALL_LINES = 1.0
MAX_PROSE_ID_SHARE = 10.0
MAX_SENTENCE_WORDS = 25.0
MAX_TICKS_GROWTH = 0.5


def clean(text):
    return FENCE.sub("", FRONT_MATTER.sub("", text))


def content_lines(text):
    for line in clean(text).splitlines():
        stripped = line.strip()
        if stripped and not stripped.startswith("#") and not re.fullmatch(r"[|\-: ]+", stripped):
            yield stripped


def count_words(text):
    return len(re.findall(r"\S+", text))


def share(part, total):
    return round(100 * part / total, 1) if total else 0.0


def measure(text):
    lines = list(content_lines(text))
    prose = [line for line in lines if not line.startswith("|")]
    sentences = [s for line in prose for s in SENTENCE_END.split(LIST_MARK.sub("", line)) if count_words(s)]
    words = sum(count_words(line) for line in lines)
    return {
        "linhas": len(lines),
        "pct_linhas_com_id": share(sum(1 for l in lines if ID.search(l)), len(lines)),
        "linhas_de_texto": len(prose),
        "pct_texto_com_id": share(sum(1 for l in prose if ID.search(l)), len(prose)),
        "frase_media": round(sum(count_words(s) for s in sentences) / len(sentences), 1),
        "palavras": words,
        "crases_por_100_palavras": round(100 * sum(l.count("`") // 2 for l in lines) / words, 1),
    }


def defined_ids(text):
    return set(DEFINED_ID.findall(clean(text)))


def compare(before_text, after_text):
    before, after = measure(before_text), measure(after_text)
    missing = sorted(defined_ids(before_text) - defined_ids(after_text))
    checks = [
        ("linhas com ID: queda de pelo menos %g ponto percentual" % MIN_DROP_ALL_LINES,
         before["pct_linhas_com_id"] - after["pct_linhas_com_id"] >= MIN_DROP_ALL_LINES),
        ("texto corrido com ID: no máximo %g%% das linhas" % MAX_PROSE_ID_SHARE, after["pct_texto_com_id"] <= MAX_PROSE_ID_SHARE),
        ("frase média: no máximo %g palavras" % MAX_SENTENCE_WORDS, after["frase_media"] <= MAX_SENTENCE_WORDS),
        ("crases por 100 palavras: alta de no máximo %g" % MAX_TICKS_GROWTH,
         after["crases_por_100_palavras"] - before["crases_por_100_palavras"] <= MAX_TICKS_GROWTH),
        ("nenhum requisito, regra, caso de teste ou borda perdido", not missing),
    ]
    return before, after, checks, missing


def main(argv):
    if len(argv) == 2:
        for key, value in measure(open(argv[1], encoding="utf-8").read()).items():
            print(f"{key}: {value}")
        return 0
    before, after, checks, missing = compare(*(open(p, encoding="utf-8").read() for p in argv[1:3]))
    print(f"{'medida':<28}{'antes':>10}{'depois':>10}")
    for key in before:
        print(f"{key:<28}{before[key]:>10}{after[key]:>10}")
    for label, passed in checks:
        print(f"{'OK' if passed else 'FALHOU'}: {label}")
    if missing:
        print("IDs perdidos: " + ", ".join(missing))
    return 0 if all(passed for _, passed in checks) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv) if len(sys.argv) in (2, 3) else print(__doc__) or 2)
