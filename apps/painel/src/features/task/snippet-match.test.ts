import { describe, expect, it } from 'vitest';
import { findLines, stripAnsi } from './snippet-match';

const HEADER = '$ cmd\n# data: d\n# commit: c\n# código de saída: 0\n';
const log = (...lines: string[]) => HEADER + lines.join('\n');

describe('findLines', () => {
  describe('Falsos positivos', () => {
    it('não casa 0 passed dentro de 10 passed', () => {
      expect(findLines(log('Tests  10 passed (10)'), '0 passed')).toEqual([]);
    });

    it('não casa a palavra em maiúsculas com o resumo em minúsculas do cargo', () => {
      expect(findLines(log('test result: ok. 3 passed; 0 failed; 0 ignored'), 'FAILED')).toEqual([]);
    });

    it('não casa um prefixo dentro de uma palavra maior', () => {
      expect(findLines(log('test busca::acha ... FAILED'), 'FAIL')).toEqual([]);
    });

    it('não casa o singular dentro do plural', () => {
      expect(findLines(log('Found 0 errors.'), 'error')).toEqual([]);
    });

    it('não casa o sinal de vezes colado a números', () => {
      expect(findLines(log('print 1280×720 gravado', 'zoom 2×'), '×')).toEqual([]);
    });

    it('não casa ok dentro de token', () => {
      expect(findLines(log('token da sessão gerado'), 'ok')).toEqual([]);
    });

    it('não casa o texto que só aparece no cabeçalho do log', () => {
      const text = '$ cargo test\n# data: d\n# commit: abc1234\n# código de saída: 0\ntest result: ok';
      expect(findLines(text, 'cargo test')).toEqual([]);
      expect(findLines(text, '# commit')).toEqual([]);
      expect(findLines(text, 'abc1234')).toEqual([]);
    });
  });

  describe('Regras da busca', () => {
    it('diferencia maiúsculas', () => {
      const text = log('Test Files  1 passed (1)');
      expect(findLines(text, 'test files')).toEqual([]);
      expect(findLines(text, 'Test Files')).toEqual([4]);
    });

    it('junta os espaços repetidos do log e do trecho', () => {
      expect(findLines(log('Test Files  1 passed (1)'), 'Test   Files 1 passed')).toEqual([4]);
    });

    it('prefere a linha igual à linha que contém o texto', () => {
      expect(findLines(log('test result: ok. 3 passed', 'test result: ok'), 'test result: ok')).toEqual([5]);
      expect(findLines(log('test result: ok. 3 passed', 'Tests 2 passed'), 'passed')).toEqual([4, 5]);
    });

    it('exige palavra inteira com acentos como letras', () => {
      const text = log('ação concluída', 'não');
      expect(findLines(text, 'ação')).toEqual([4]);
      expect(findLines(text, 'ção')).toEqual([]);
      expect(findLines(text, 'ão')).toEqual([]);
    });

    it('casa o sinal de vezes do Vitest como palavra', () => {
      const text = log(' × src/a.test.ts > busca > acha 3ms', ' ✓ src/a.test.ts > busca > lê 2ms');
      expect(findLines(text, '×')).toEqual([4]);
    });

    it('aceita o curinga no começo e no fim sem limite de palavra junto dele', () => {
      const text = log('Tests  10 passed (10)', 'Tests  0 passed (0)', 'Test Files  1 passed');
      expect(findLines(text, '... passed')).toEqual([4, 5, 6]);
      expect(findLines(text, '... 0 passed')).toEqual([4, 5]);
      expect(findLines(text, 'Test Fi ...')).toEqual([6]);
      expect(findLines(text, '... passe')).toEqual([]);
      expect(findLines(text, 'Tests ... 0 passed')).toEqual([4, 5]);
    });

    it('aceita o curinga em caractere único e exige as partes na ordem', () => {
      const text = log('✓ src/board.test.tsx > Board > Filtro > esconde a raia');
      expect(findLines(text, '✓ … Board … esconde')).toEqual([4]);
      expect(findLines(text, 'esconde ... Board')).toEqual([]);
    });

    it('busca cada texto dentro de uma linha só', () => {
      expect(findLines(log('Barra da onda', 'abre a próxima'), 'Barra ... abre')).toEqual([]);
    });

    it('não casa um texto vazio ou só de curingas', () => {
      for (const snippet of ['', '   ', '...', '…', ' ... … ']) {
        expect(findLines(log('qualquer linha'), snippet)).toEqual([]);
      }
    });

    it('ignora os códigos de cor ANSI', () => {
      const text = log(
        '\u001b[32m✓\u001b[39m src/a.test.ts > busca > passa \u001b[2m3ms\u001b[22m',
        '      Tests  \u001b[1m\u001b[32m2 passed\u001b[39m\u001b[22m (2)',
      );
      expect(findLines(text, '✓ ... > busca > passa')).toEqual([4]);
      expect(findLines(text, 'Tests 2 passed (2)')).toEqual([5]);
      expect(findLines(text, '32m')).toEqual([]);
    });

    it('devolve os índices das linhas do log original', () => {
      expect(findLines('linha sem cabeçalho\nok', 'ok')).toEqual([1]);
    });
  });

  describe('Caso real da SPEC-049 TASK-09', () => {
    it('acha os quatro textos do Trecho da EV-02 nas linhas 83, 86, 87 e 90', () => {
      const cases: Array<[string, number]> = [
        ['✓ ... Barra da onda > abre a próxima com → e a anterior com ←', 83],
        ['✓ ... Próxima a conferir > pula para a próxima tarefa da onda com evidência a conferir', 86],
        ['... > avisa e fica na tarefa quando nenhuma tem evidência a conferir', 87],
        ['Test Files  1 passed', 90],
      ];
      for (const [snippet, line] of cases) {
        expect(findLines(BARRA_DA_ONDA, snippet).map((index) => index + 1)).toEqual([line]);
      }
    });

    it('não acha os textos da Falha da EV-02', () => {
      expect(findLines(BARRA_DA_ONDA, '×')).toEqual([]);
      expect(findLines(BARRA_DA_ONDA, 'failed')).toEqual([]);
    });
  });
});

describe('stripAnsi', () => {
  it('tira os códigos de cor e mantém o texto', () => {
    expect(stripAnsi('\u001b[1m\u001b[32m2 passed\u001b[39m\u001b[22m')).toBe('2 passed');
    expect(stripAnsi('sem cor')).toBe('sem cor');
  });
});

const BARRA_DA_ONDA = `$ pnpm --dir apps/painel exec vitest run src/features/task/task-page.test.tsx --reporter=verbose
# data: 2026-11-11T19:46:19Z
# commit: 8047718 (com mudanças não commitadas)
# código de saída: 0


 RUN  v4.1.12 /home/victor/projetos/ai-hub/apps/painel

 ✓ src/features/task/task-page.test.tsx > TaskPage > divide a tarefa nas abas Resumo, Evidências e Para o agente 274ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > esconde onda, perfil e persona vazios 32ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > mostra onda, perfil e persona com valor 21ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > shows image evidence 45ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > não oferece Editar em nenhuma parte 60ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > shows the empty evidence note and the error state 38ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > estado da conferência > mostra os quatro estados da conferência 44ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > estado da conferência > mostra o motivo da rejeição 25ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > estado da conferência > não mostra botão de conferir nem de rejeitar 66ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > estado da conferência > sem reviews não mostra selo de conferência 32ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > estado da conferência > mostra o selo da evidência manual anexada 49ms
 ✓ src/features/task/task-page.test.tsx > TaskPage > não mostra botão de ação em nenhum status 88ms
 ✓ src/features/task/task-page.test.tsx > resposta real da API > mostra a tarefa quando a resposta não traz manual_evidence 20ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > mostra imagem, vídeo e texto e só dá o link aos demais 136ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > mantém a conferência no print declarado como svg 52ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > pede só o primeiro 1 MB do texto e avisa quando há mais 74ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > mostra inteiro o texto menor, sem aviso 56ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > aceita a resposta sem Content-Range 44ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > mostra o caractere de substituição no texto cortado 43ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > não pede o conteúdo de um texto vazio 65ms
 ✓ src/features/task/task-page.test.tsx > Evidências por tipo > mostra a falha ao ler o texto 61ms
 ✓ src/features/task/task-page.test.tsx > anexo de evidência > mostra o que o harness pede, sem Anexar 45ms
 ✓ src/features/task/task-page.test.tsx > anexo de evidência > mostra a imagem, sem Trocar 50ms
 ✓ src/features/task/task-page.test.tsx > anexo de evidência > mostra o link de baixar, sem Trocar 55ms
 ✓ src/features/task/task-page.test.tsx > anexo de evidência > não repete a evidência manual na lista geral 17ms
 ✓ src/features/task/task-page.test.tsx > anexo de evidência > abre a aba Evidências com #evidencias 31ms
 ✓ src/features/task/task-page.test.tsx > TaskPage em abas > guarda a aba aberta no hash 17ms
 ✓ src/features/task/task-page.test.tsx > TaskPage em abas > abre Resumo com hash desconhecido 18ms
 ✓ src/features/task/task-page.test.tsx > TaskPage em abas > não mostra o índice nem os cards e mantém o aside 49ms
 ✓ src/features/task/task-page.test.tsx > ordem da evidência > mostra a evidência na ordem de leitura 33ms
 ✓ src/features/task/task-page.test.tsx > ordem da evidência > mostra o nome do arquivo no lugar do título na evidência antiga 34ms
 ✓ src/features/task/task-page.test.tsx > ordem da evidência > não mostra a linha de conferência sem reviews 30ms
 ✓ src/features/task/task-page.test.tsx > ordem da evidência > mostra só o nome do arquivo para evidência fora da verificação 30ms
 ✓ src/features/task/task-page.test.tsx > Esperas no aside > mostra Espera, Fecha depois de e Depende de no aside 13ms
 ✓ src/features/task/task-page.test.tsx > Esperas no aside > esconde Fecha depois de quando a tarefa já está concluída 13ms
 ✓ src/features/task/task-page.test.tsx > Esperas no aside > não mostra espera sem waiting_on 11ms
 ✓ src/features/task/task-page.test.tsx > Componentes de mídia na evidência > abre a imagem da evidência no visualizador 62ms
 ✓ src/features/task/task-page.test.tsx > Componentes de mídia na evidência > toca o vídeo da evidência no player 43ms
 ✓ src/features/task/task-page.test.tsx > Componentes de mídia na evidência > copia o conteúdo inteiro do texto com o cabeçalho 75ms
 ✓ src/features/task/task-page.test.tsx > Componentes de mídia na evidência > não mostra o botão de copiar antes de o texto carregar 42ms
 ✓ src/features/task/task-page.test.tsx > ID das evidências > mostra o título com o ID como prefixo 29ms
 ✓ src/features/task/task-page.test.tsx > ID das evidências > mostra o ID com o nome do arquivo sem Título 25ms
 ✓ src/features/task/task-page.test.tsx > ID das evidências > mostra o título sem prefixo quando a resposta não traz o ID 25ms
 ✓ src/features/task/task-page.test.tsx > Filtro de evidências > filtra por todas, a conferir e conferidas com a contagem 99ms
 ✓ src/features/task/task-page.test.tsx > Filtro de evidências > avisa quando nenhuma evidência está no filtro 36ms
 ✓ src/features/task/task-page.test.tsx > Filtro de evidências > não mostra o filtro sem evidências 15ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > prefere a linha igual à linha que contém o texto 1ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > usa a linha que contém o texto quando nenhuma é igual 2ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > normaliza os espaços na busca 2ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > destaca todas as linhas do trecho genérico 3ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > ignora as quatro linhas de cabeçalho do log 5ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > devolve vazio para o trecho que não aparece 3ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > reticências no trecho > casa os textos da TASK-08 com reticências no meio 3ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > reticências no trecho > casa os textos da EV-02 da TASK-08 com o tempo no fim da linha 7ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > reticências no trecho > aceita as reticências em caractere único 2ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > reticências no trecho > casa as reticências no início do texto 1ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > reticências no trecho > mantém o limite de palavra só nas pontas sem reticências 5ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > reticências no trecho > não casa quando as partes aparecem fora de ordem 2ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > reticências no trecho > não casa um texto só de reticências 0ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > limite de palavra > não casa o texto no meio de um número 0ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > limite de palavra > casa o texto no fim de uma linha com outras palavras 0ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > limite de palavra > não casa dentro de uma palavra 2ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > limite de palavra > respeita acentos como letras 1ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > cópia do trecho > copia o texto do trecho sem as crases ao clicar e confirma por cerca de 1,5 s 1561ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > cópia do trecho > copia também o texto da Falha 44ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > cópia do trecho > não copia ao passar o mouse nem ao focar 42ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > cópia do trecho > avisa Não foi possível copiar e mantém o destaque quando a cópia é recusada 62ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > cópia do trecho > avisa Não foi possível copiar quando não há área de transferência 41ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > abre o log, destaca as linhas e rola até a primeira ao passar o mouse 114ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > destaca também pelo foco do teclado 50ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > tira o destaque ao sair do trecho 60ms
 ✓ src/features/task/task-page.test.tsx > Destaque do Trecho > marca o trecho não encontrado como alerta e a falha não encontrada como bom 61ms
 ✓ src/features/task/task-page.test.tsx > Barra da onda > volta por padrão ao quadro filtrado pela spec 29ms
 ✓ src/features/task/task-page.test.tsx > Barra da onda > mostra a anterior, a próxima e a posição na onda 16ms
 ✓ src/features/task/task-page.test.tsx > Barra da onda > abre a próxima com → e a anterior com ← 62ms
 ✓ src/features/task/task-page.test.tsx > Barra da onda > não troca de tarefa com as setas nas abas, num campo ou com modificador 16ms
 ✓ src/features/task/task-page.test.tsx > Barra da onda > mostra só a posição na tarefa única na onda ou sem onda 19ms
 ✓ src/features/task/task-page.test.tsx > Próxima a conferir > pula para a próxima tarefa da onda com evidência a conferir 22ms
 ✓ src/features/task/task-page.test.tsx > Próxima a conferir > avisa e fica na tarefa quando nenhuma tem evidência a conferir 21ms
 ✓ src/features/task/task-page.test.tsx > Próxima a conferir > pula a tarefa que não abre e segue para a próxima 14ms

 Test Files  1 passed (1)
      Tests  80 passed (80)
   Start at  16:46:13
   Duration  5.52s (transform 320ms, setup 0ms, import 445ms, tests 4.50s, environment 436ms)

`;
