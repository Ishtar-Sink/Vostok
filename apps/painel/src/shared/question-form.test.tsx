import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { parseQuestionForm, QuestionForm } from './question-form';

afterEach(cleanup);

const HINT = 'Responda com o número e a letra (ex.: 1A, 2B). Caso nenhuma se encaixe, descreva como deseja prosseguir.';
const P002 = [
  '## Pergunta', '', '# 1. Stack do app', '', 'Escolha: única', '',
  'O repositório só tem o harness, e nada define a linguagem.', '',
  '| Opção | Descrição |', '|---|---|',
  '| A | Web com TypeScript, React e Vite (recomendada) |',
  '| B | Web com HTML e JavaScript puros |',
  '| C | Outra stack; informe a linguagem |', '',
  HINT, '',
  '# 2. A lista de tarefas com filtros ainda não existe', '', 'Escolha: várias', '',
  'A spec assume que a lista e os filtros já existem.', '',
  '| Opção | Descrição |', '|---|---|',
  '| A | Criar antes uma spec para a lista (recomendada) |',
  '| B | Ajustar a SPEC-001 para incluir uma lista mínima |',
  '| C | Aprovar agora, registrando o desvio |', '',
  HINT,
].join('\n');

const block = (n: number, title: string) =>
  [`# ${n}. ${title}`, '', 'Contexto.', '', '| Opção | Descrição |', '|---|---|', '| A | Um |', '| B | Dois |', '', HINT, ''].join('\n');
const P001_SEM_ESCOLHA = ['## Pergunta', '', block(1, 'Primeira'), block(2, 'Segunda'), block(3, 'Terceira'), '## Inferências', '', '# 9. Não é pergunta', '', '- algo'].join('\n');


const REAL = [
  '## Pergunta', '', '## O que aconteceu', '', 'A exportação parou na cópia.', '',
  '## Efeito de cada opção', '', '| Opção | Efeito |', '|---|---|', '| A | muda tudo |', '',
  '# 1. Como tratar a linha', '', 'Escolha: única', '', 'A linha é a versão do Java.', '',
  '| Item | Nota |', '|---|---|', '| A | outra |', '',
  '| Opção | Descrição |', '|---|---|', '| A | Aceitar (recomendada) |', '| B | Mascarar |', '',
  '## Antes da segunda', '', 'Vale para todas as cópias.', '',
  '# 2. O que mais incluir', '', 'Escolha: várias', '',
  '| Opção | Descrição |', '|---|---|', '| A | Manifesto |', '| B | Hashes |', '',
  HINT, '',
  '## Inferências', '', '- algo', '', '## Resposta', '', 'texto',
].join('\n');

describe('parseQuestionForm', () => {
  it('lê as perguntas do template com a escolha de cada uma', () => {
    const questions = parseQuestionForm(P002).questions;
    expect(questions).toHaveLength(2);
    expect(questions[0]).toMatchObject({ number: 1, title: 'Stack do app', multiple: false });
    expect(questions[0].options.map((option) => option.letter)).toEqual(['A', 'B', 'C']);
    expect(questions[1].multiple).toBe(true);
    expect(questions[0].options[0].description).toBe('Web com TypeScript, React e Vite (recomendada)');
    expect(questions[0].context).toContain('nada define a linguagem');
    for (const out of ['Escolha:', '| A |', 'Responda com']) expect(questions[0].context).not.toContain(out);
  });

  it('trata a pergunta sem a linha Escolha como única', () => {
    const questions = parseQuestionForm(P001_SEM_ESCOLHA).questions;
    expect(questions).toHaveLength(3);
    expect(questions.every((question) => !question.multiple)).toBe(true);
    expect(questions.some((question) => question.title.includes('Não é pergunta'))).toBe(false);
  });

  it('não lê a pergunta fora do template', () => {
    expect(parseQuestionForm('## Pergunta\nQual caminho seguir?\n\n## Opções\n- A\n- B\n').questions).toEqual([]);
    expect(parseQuestionForm('# 1. Sem tabela\n\nTexto sem opções.').questions).toEqual([]);
  });

  it('lê a pergunta inteira com títulos de nível 2 antes e entre as perguntas', () => {
    const form = parseQuestionForm(REAL);
    expect(form.questions).toHaveLength(2);
    expect(form.questions[0].options.map((option) => option.letter)).toEqual(['A', 'B']);
    expect(form.questions[1].multiple).toBe(true);
    expect(form.questions[0].context).toContain('Antes da segunda');
  });

  it('para em Inferências e em Resposta', () => {
    const form = parseQuestionForm(REAL);
    const all = [form.intro, ...form.questions.map((question) => question.context)].join('\n');
    expect(all).not.toContain('algo');
    expect(all).not.toContain('texto');
    const semInferencias = REAL.replace('## Inferências\n\n- algo\n\n', '');
    expect(parseQuestionForm(semInferencias).questions[1].context).not.toContain('texto');
  });

  it('tira as opções só da tabela Opção e Descrição', () => {
    const form = parseQuestionForm(REAL);
    expect(form.questions[0].options).toEqual([
      { letter: 'A', description: 'Aceitar (recomendada)' },
      { letter: 'B', description: 'Mascarar' },
    ]);
    expect(form.questions[0].context).toContain('| A | outra |');
    expect(form.questions[0].context).not.toContain('Aceitar');
    expect(form.intro).toContain('muda tudo');
  });

  it('separa o texto inicial do contexto de cada pergunta', () => {
    const form = parseQuestionForm(REAL);
    expect(form.intro).toContain('A exportação parou na cópia.');
    expect(form.questions[0].context).toContain('A linha é a versão do Java.');
    expect(form.questions[0].context).not.toContain('A exportação parou');
    expect(form.questions[0].context).not.toContain('Escolha:');
  });

  it('não lê a pergunta sem tabela de opções em alguma pergunta', () => {
    const md = ['## Pergunta', '', block(1, 'Primeira'), '# 2. Segunda', '', 'Sem tabela.', ''].join('\n');
    expect(parseQuestionForm(md)).toEqual({ intro: '', questions: [] });
  });
});

describe('QuestionForm', () => {
  const setup = (sending = false) => {
    const onSubmit = vi.fn();
    render(<QuestionForm questions={parseQuestionForm(P002).questions} sending={sending} onSubmit={onSubmit} />);
    return onSubmit;
  };
  const group = (name: string) => within(screen.getByRole('group', { name }));
  const g1 = '1. Stack do app';
  const g2 = '2. A lista de tarefas com filtros ainda não existe';
  const send = () => fireEvent.click(screen.getByRole('button', { name: 'Enviar resposta' }));

  it('mostra radio na escolha única e checkbox na várias', () => {
    setup();
    expect(group(g1).getAllByRole('radio')).toHaveLength(4);
    expect(group(g1).queryAllByRole('checkbox')).toHaveLength(0);
    expect(group(g2).getAllByRole('checkbox')).toHaveLength(4);
    expect(group(g2).queryAllByRole('radio')).toHaveLength(0);
    expect(screen.getByRole('textbox', { name: 'Observações' })).toBeTruthy();
  });

  it('monta a resposta no formato 1A, 2B', () => {
    const onSubmit = setup();
    fireEvent.click(group(g1).getByRole('radio', { name: /^A / }));
    fireEvent.click(group(g2).getByRole('checkbox', { name: /^B / }));
    send();
    expect(onSubmit).toHaveBeenCalledWith('1A, 2B');
  });

  it('junta as várias escolhas e as observações', () => {
    const onSubmit = setup();
    fireEvent.click(group(g1).getByRole('radio', { name: /^A / }));
    fireEvent.click(group(g2).getByRole('checkbox', { name: /^C / }));
    fireEvent.click(group(g2).getByRole('checkbox', { name: /^A / }));
    fireEvent.input(screen.getByRole('textbox', { name: 'Observações' }), { target: { value: 'Usar pnpm' } });
    send();
    expect(onSubmit).toHaveBeenCalledWith('1A, 2A, 2C\n\nObservações: Usar pnpm');
  });

  it('só libera o envio com todas respondidas ou com observações', () => {
    const onSubmit = setup();
    const button = screen.getByRole('button', { name: 'Enviar resposta' }) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    fireEvent.click(group(g1).getByRole('radio', { name: /^A / }));
    expect(button.disabled).toBe(true);
    fireEvent.input(screen.getByRole('textbox', { name: 'Observações' }), { target: { value: 'Prefiro outra ordem' } });
    expect(button.disabled).toBe(false);
    send();
    expect(onSubmit).toHaveBeenCalledWith('1A\n\nObservações: Prefiro outra ordem');
  });

  it('desliga o envio durante o envio', () => {
    setup(true);
    fireEvent.click(group(g1).getByRole('radio', { name: /^A / }));
    fireEvent.click(group(g2).getByRole('checkbox', { name: /^B / }));
    expect((screen.getByRole('button', { name: /Enviando/ }) as HTMLButtonElement).disabled).toBe(true);
  });

  describe('Outro', () => {
    const outro = (name: string) => group(name).getByRole('radio', { name: 'Outro' }) as HTMLInputElement;
    const outroCheck = (name: string) => group(name).getByRole('checkbox', { name: 'Outro' }) as HTMLInputElement;
    const field = (n: number) => screen.queryByRole('textbox', { name: `Outro da pergunta ${n}` }) as HTMLInputElement | null;
    const button = () => screen.getByRole('button', { name: 'Enviar resposta' }) as HTMLButtonElement;

    it('mostra o texto inicial e o contexto acima das opções', () => {
      const form = parseQuestionForm(REAL);
      render(<QuestionForm intro={form.intro} questions={form.questions} sending={false} onSubmit={vi.fn()} />);
      const intro = screen.getByText('A exportação parou na cópia.');
      const context = screen.getByText('A linha é a versão do Java.');
      const first = screen.getAllByRole('radio')[0];
      expect(intro.compareDocumentPosition(context) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      expect(context.compareDocumentPosition(first) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    });

    it('acrescenta Outro em escolha única e múltipla', () => {
      setup();
      expect(outro(g1)).toBeTruthy();
      expect(outroCheck(g2)).toBeTruthy();
    });

    it('só mostra o campo de Outro com a opção marcada', () => {
      setup();
      expect(field(1)).toBeNull();
      fireEvent.click(outro(g1));
      expect(field(1)).not.toBeNull();
      fireEvent.click(outroCheck(g2));
      expect(field(2)).not.toBeNull();
      fireEvent.click(outroCheck(g2));
      expect(field(2)).toBeNull();
    });

    it('troca Outro por uma letra na escolha única', () => {
      const onSubmit = setup();
      fireEvent.click(outro(g1));
      fireEvent.input(field(1)!, { target: { value: 'Usar a pasta' } });
      fireEvent.click(group(g1).getByRole('radio', { name: /^A / }));
      expect(field(1)).toBeNull();
      fireEvent.click(group(g2).getByRole('checkbox', { name: /^B / }));
      send();
      expect(onSubmit).toHaveBeenCalledWith('1A, 2B');
      fireEvent.click(outro(g1));
      expect((group(g1).getByRole('radio', { name: /^A / }) as HTMLInputElement).checked).toBe(false);
    });

    it('bloqueia o envio com Outro vazio, mesmo com observações', () => {
      setup();
      fireEvent.click(group(g1).getByRole('radio', { name: /^A / }));
      fireEvent.click(outroCheck(g2));
      fireEvent.input(screen.getByRole('textbox', { name: 'Observações' }), { target: { value: 'Sem pressa' } });
      expect(button().disabled).toBe(true);
      fireEvent.input(field(2)!, { target: { value: '  ' } });
      expect(button().disabled).toBe(true);
    });

    it('põe Outro em linha própria, depois das letras', () => {
      const onSubmit = setup();
      fireEvent.click(group(g1).getByRole('radio', { name: /^A / }));
      fireEvent.click(group(g2).getByRole('checkbox', { name: /^B / }));
      fireEvent.click(outroCheck(g2));
      fireEvent.input(field(2)!, { target: { value: 'Usar a pasta' } });
      fireEvent.input(screen.getByRole('textbox', { name: 'Observações' }), { target: { value: 'Sem pressa' } });
      send();
      expect(onSubmit).toHaveBeenCalledWith('1A, 2B\n2 Outro: Usar a pasta\n\nObservações: Sem pressa');
    });

    it('aceita a resposta só com Outro', () => {
      const onSubmit = setup();
      fireEvent.click(outro(g1));
      fireEvent.input(field(1)!, { target: { value: 'Um' } });
      fireEvent.click(outroCheck(g2));
      fireEvent.input(field(2)!, { target: { value: 'Dois' } });
      send();
      expect(onSubmit).toHaveBeenCalledWith('1 Outro: Um\n2 Outro: Dois');
    });

    it('nomeia o campo de Outro para o leitor de tela', () => {
      setup();
      fireEvent.click(outro(g1));
      expect(screen.getByRole('textbox', { name: 'Outro da pergunta 1' })).toBeTruthy();
    });
  });
});
