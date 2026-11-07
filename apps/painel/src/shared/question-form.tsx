import { useState } from 'react';
import { Button } from '../ui/button/button';
import { Checkbox } from '../ui/checkbox/checkbox';
import { Markdown } from './markdown';
import styles from './question-form.module.css';

export interface QuestionOption {
  letter: string;
  description: string;
}

export interface ParsedQuestion {
  number: number;
  title: string;
  context: string;
  multiple: boolean;
  options: QuestionOption[];
}

const TABLE_HEADER = /^\|\s*Opção\s*\|\s*Descrição\s*\|\s*$/i;
const OPTION_ROW = /^\|\s*([A-Z])\s*\|\s*(.+?)\s*\|\s*$/;

/** Corpo da pergunta: depois de `## Pergunta` até `## Inferências` ou `## Resposta`, sem parar nos outros `## `. */
export function questionBody(markdown: string): string {
  const lines = markdown.split('\n');
  const start = lines.findIndex((line) => /^## Pergunta\s*$/.test(line));
  const rest = start >= 0 ? lines.slice(start + 1) : lines;
  const end = rest.findIndex((line) => /^## (Inferências|Resposta)\s*$/.test(line));
  return (end >= 0 ? rest.slice(0, end) : rest).join('\n');
}

function readOptions(lines: string[]): { options: QuestionOption[]; rest: string[] } {
  const header = lines.findIndex((line) => TABLE_HEADER.test(line));
  if (header < 0) return { options: [], rest: lines };
  const options: QuestionOption[] = [];
  let end = header + 2;
  for (; end < lines.length; end++) {
    const match = OPTION_ROW.exec(lines[end]);
    if (!match) break;
    options.push({ letter: match[1], description: match[2] });
  }
  return { options, rest: [...lines.slice(0, header), ...lines.slice(end)] };
}

/** Lê o texto inicial e as perguntas do template `question`; fora dele devolve vazio e a pergunta segue em texto livre (CB-03). */
export function parseQuestionForm(markdown: string): { intro: string; questions: ParsedQuestion[] } {
  const empty = { intro: '', questions: [] };
  const introLines: string[] = [];
  const blocks: { number: number; title: string; lines: string[] }[] = [];
  for (const line of questionBody(markdown).split('\n')) {
    const heading = /^# (\d+)\. (.+)$/.exec(line);
    if (heading) blocks.push({ number: Number(heading[1]), title: heading[2].trim(), lines: [] });
    else (blocks.at(-1)?.lines ?? introLines).push(line);
  }
  if (blocks.length === 0) return empty;
  const questions: ParsedQuestion[] = [];
  for (const block of blocks) {
    const { options, rest } = readOptions(block.lines);
    if (options.length === 0) return empty;
    const context = rest.filter((line) => !/^Escolha:/i.test(line) && !line.startsWith('Responda com')).join('\n').trim();
    questions.push({ number: block.number, title: block.title, context, multiple: block.lines.some((line) => /^Escolha:\s*várias\s*$/i.test(line)), options });
  }
  return { intro: introLines.join('\n').trim(), questions };
}

export function QuestionForm({ intro, questions, notes, sending, onSubmit }: { intro?: string; questions: ParsedQuestion[]; notes?: string; sending: boolean; onSubmit: (answer: string) => void }) {
  const [choices, setChoices] = useState<Record<number, string[]>>({});
  const [others, setOthers] = useState<Record<number, { on: boolean; text: string }>>({});
  const [observations, setObservations] = useState('');
  const otherOf = (number: number) => others[number] ?? { on: false, text: '' };
  const picked = questions.flatMap((question) => [...(choices[question.number] ?? [])].sort().map((letter) => `${question.number}${letter}`));
  const otherLines = questions.filter((question) => otherOf(question.number).on).map((question) => `${question.number} Outro: ${otherOf(question.number).text.trim()}`);
  const answer = [[picked.join(', '), ...otherLines].filter(Boolean).join('\n'), observations.trim() ? `Observações: ${observations.trim()}` : ''].filter(Boolean).join('\n\n');
  const otherEmpty = questions.some((question) => otherOf(question.number).on && otherOf(question.number).text.trim() === '');
  const answered = questions.every((question) => (choices[question.number] ?? []).length > 0 || otherOf(question.number).on);
  const ready = !otherEmpty && (answered || observations.trim() !== '');

  function setOther(number: number, patch: Partial<{ on: boolean; text: string }>) {
    setOthers((current) => ({ ...current, [number]: { ...(current[number] ?? { on: false, text: '' }), ...patch } }));
  }

  function toggle(number: number, letter: string, on: boolean) {
    setChoices((current) => {
      const has = current[number] ?? [];
      return { ...current, [number]: on ? [...has.filter((item) => item !== letter), letter] : has.filter((item) => item !== letter) };
    });
  }

  function chooseLetter(question: ParsedQuestion, letter: string) {
    setChoices((current) => ({ ...current, [question.number]: [letter] }));
    setOther(question.number, { on: false });
  }

  function chooseOther(question: ParsedQuestion) {
    setChoices((current) => ({ ...current, [question.number]: [] }));
    setOther(question.number, { on: true });
  }

  return (
    <form
      className={styles.form}
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit(answer);
      }}
    >
      {intro ? (
        <div className={styles.intro}>
          <Markdown source={intro} />
        </div>
      ) : null}
      {questions.map((question) => (
        <fieldset key={question.number} className={styles.question}>
          <legend>{`${question.number}. ${question.title}`}</legend>
          {question.context ? <Markdown source={question.context} /> : null}
          {question.options.map((option) =>
            question.multiple ? (
              <Checkbox
                key={option.letter}
                label={`${option.letter} ${option.description}`}
                checked={(choices[question.number] ?? []).includes(option.letter)}
                onChange={(event) => toggle(question.number, option.letter, (event.currentTarget as HTMLInputElement).checked)}
              />
            ) : (
              <label key={option.letter} className={styles.option}>
                <input
                  type="radio"
                  name={`pergunta-${question.number}`}
                  value={option.letter}
                  checked={(choices[question.number] ?? [])[0] === option.letter}
                  onChange={() => chooseLetter(question, option.letter)}
                />
                <span>
                  <span className={styles.letter}>{option.letter}</span> {option.description}
                </span>
              </label>
            ),
          )}
          {question.multiple ? (
            <Checkbox label="Outro" checked={otherOf(question.number).on} onChange={(event) => setOther(question.number, { on: (event.currentTarget as HTMLInputElement).checked })} />
          ) : (
            <label className={styles.option}>
              <input type="radio" name={`pergunta-${question.number}`} value="Outro" checked={otherOf(question.number).on} onChange={() => chooseOther(question)} />
              <span>Outro</span>
            </label>
          )}
          {otherOf(question.number).on ? (
            <input
              type="text"
              className={styles.other}
              aria-label={`Outro da pergunta ${question.number}`}
              value={otherOf(question.number).text}
              onInput={(event) => setOther(question.number, { text: (event.currentTarget as HTMLInputElement).value })}
            />
          ) : null}
        </fieldset>
      ))}
      {notes ? <Markdown source={notes} /> : null}
      <label className={styles.notes}>
        <span>Observações</span>
        <textarea rows={3} value={observations} onInput={(event) => setObservations((event.currentTarget as HTMLTextAreaElement).value)} />
      </label>
      <Button type="submit" variant="primary" className={styles.submit} disabled={!ready || sending}>
        {sending ? 'Enviando…' : 'Enviar resposta'}
      </Button>
    </form>
  );
}
