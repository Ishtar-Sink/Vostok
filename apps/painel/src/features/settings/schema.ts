/** Como cada opção de `config/*.yml` aparece na tela: nome, controle e seção. O que não estiver
 *  aqui vai para "Avançado", com o nome derivado da chave. */
export interface Option {
  value: string;
  label: string;
  help?: string;
}

export type Control =
  | { type: 'text'; mono?: boolean; placeholder?: string }
  | { type: 'path' }
  | { type: 'number'; min?: number; max?: number; unit?: string }
  | { type: 'range'; min: number; max: number; unit?: string }
  | { type: 'select'; options: Option[]; empty?: string }
  | { type: 'segmented'; options: Option[] }
  | { type: 'color'; empty?: string }
  | { type: 'tags'; placeholder?: string }
  | { type: 'paths' }
  | { type: 'checklist'; options: Option[]; /** Lista de outro arquivo que limita as opções (`arquivo:chave`). */ within?: string }
  | { type: 'combos'; options: Option[] }
  | { type: 'textarea'; mono?: boolean }
  | { type: 'font' }
  | { type: 'id'; digits: string; sequence?: { file: string; key: string } };

export interface Field {
  /** Caminho com pontos; `{item}` é trocado pela opção escolhida no seletor da seção. */
  key: string;
  label: string;
  help?: string;
  control: Control;
  wide?: boolean;
}

export interface Section {
  title: string;
  description?: string;
  fields: Field[];
  /** Seletor no topo da seção, que troca o `{item}` das chaves. `fromKeys` lista as chaves sob um grupo. */
  switcher?: { label: string; options?: Option[]; fromKeys?: string; style?: 'segmented' | 'select' };
  /** Caixa que liga a seção: desligada, os campos ficam vazios. */
  /** Caixa que liga a seção, marcada quando `key` tem valor. Desligar esvazia `key`, ou todos os campos com `clearAll`. */
  toggle?: { label: string; help?: string; key: string; clearAll?: boolean };
  /** Grade de cores só de leitura, com as chaves sob `{item}` (exceto as que casam com `exclude`). */
  tokens?: { exclude: RegExp; note: string };
}

const op = (value: string, label: string, help?: string): Option => ({ value, label, help });

export const SPEC_TYPES: Option[] = [
  op('feature', 'Recurso', 'Recurso novo'),
  op('improvement', 'Melhoria', 'Melhoria de algo que já existe'),
  op('fix', 'Correção', 'Correção de defeito'),
  op('study', 'Estudo', 'Investigação, sem entrega de código'),
  op('analysis', 'Análise', 'Tese, antítese e síntese'),
];

export const SPEC_SECTIONS: Option[] = [
  op('objective', 'Objetivo'),
  op('current-state', 'Estado atual'),
  op('defect', 'Defeito'),
  op('scope', 'Escopo'),
  op('research-questions', 'Perguntas da investigação'),
  op('thesis', 'Tese'),
  op('antithesis', 'Antítese'),
  op('synthesis', 'Síntese'),
  op('functional-requirements', 'Requisitos funcionais'),
  op('non-functional-requirements', 'Requisitos não funcionais'),
  op('business-rules', 'Regras de negócio'),
  op('flows', 'Fluxos'),
  op('edge-cases', 'Casos de borda'),
  op('contracts', 'Contratos'),
  op('affected-components', 'Componentes afetados'),
  op('assumptions', 'Premissas e dependências'),
  op('risks', 'Riscos'),
  op('test-cases', 'Casos de teste'),
  op('acceptance-criteria', 'Critérios de aceite'),
  op('references', 'Referências'),
];

const PROFILES: Option[] = [
  op('mecanica', 'Mecânica', 'Passos diretos e repetitivos'),
  op('padrao', 'Padrão', 'Implementação com decisões locais'),
  op('julgamento', 'Julgamento', 'Desenho, ambiguidade e decisões amplas'),
];

const MODELS: Option[] = [
  op('haiku', 'Haiku: rápido'),
  op('sonnet', 'Sonnet: intermediário'),
  op('opus', 'Opus: o mais capaz'),
  op('sonnet[1m]', 'Sonnet com 1M de contexto'),
  op('opus[1m]', 'Opus com 1M de contexto'),
];

const FACTS: Option[] = [
  op('files_create', 'Arquivo a criar'),
  op('files_change', 'Arquivo a alterar'),
  op('ids_covered', 'ID da spec coberto'),
  op('public_contract', 'Muda contrato público'),
  op('inference', 'Depende de inferência'),
  op('high_risk', 'Cobre risco alto'),
];

const MEMORY_TYPES: Option[] = [op('decision', 'Decisões'), op('issue', 'Problemas'), op('knowledge', 'Conhecimentos')];

const facts = (prefix: string): Field[] =>
  FACTS.map((fact) => ({ key: `${prefix}.${fact.value}`, label: fact.label, help: '', control: { type: 'number', min: 0, unit: 'pontos' } }));

export const FILES: { name: string; label: string; summary: string; sections: Section[] }[] = [
  {
    name: 'main.yml',
    label: 'Geral',
    summary: 'Ferramenta de IA, repositório do harness e pastas.',
    sections: [
      {
        title: 'Ferramenta e repositório',
        fields: [
          {
            key: 'harness.tool',
            label: 'Ferramenta de IA',
            help: 'Escolhe o modelo de reserva de cada perfil quando a ferramenta não permite escolher.',
            control: { type: 'select', options: [op('claude-code', 'Claude Code'), op('codex', 'Codex'), op('cursor', 'Cursor'), op('gemini-cli', 'Gemini CLI'), op('copilot', 'GitHub Copilot')], empty: 'Modelo da sessão' },
          },
          { key: 'harness.repository', label: 'Repositório do harness', help: 'Onde os defeitos e sugestões do harness viram issues (dono/repositório no GitHub).', control: { type: 'text', mono: true, placeholder: 'dono/repositório' } },
        ],
      },
      {
        title: 'Painel',
        description: 'Avisos do painel e acompanhamento dos pedidos.',
        fields: [
          { key: 'toast_dismiss_seconds', label: 'Tempo do aviso na tela', help: 'O aviso some sozinho depois deste tempo. 0 faz ele só sumir ao ser fechado.', control: { type: 'number', min: 0, unit: 's' } },
          { key: 'request_note_warning_minutes', label: 'Aviso de falta de nota', help: 'Pedido em andamento sem nota nova por mais que isto mostra o aviso.', control: { type: 'number', min: 1, unit: 'min' } },
        ],
      },
      {
        title: 'Pastas',
        description: 'Caminhos relativos à pasta do harness (a que contém config/).',
        fields: [
          { key: 'paths.root', label: 'Raiz do projeto', control: { type: 'path' } },
          { key: 'paths.specs', label: 'Specs', control: { type: 'path' } },
          { key: 'paths.plans', label: 'Planos', control: { type: 'path' } },
          { key: 'paths.tasks', label: 'Tarefas', control: { type: 'path' } },
          { key: 'paths.archive', label: 'Specs arquivadas', control: { type: 'path' } },
          { key: 'paths.finalize', label: 'Finalização', control: { type: 'path' } },
          { key: 'paths.docs', label: 'Documentação', control: { type: 'path' } },
          { key: 'paths.spec_docs', label: 'Documentos publicados', control: { type: 'path' } },
          { key: 'paths.agent_memory', label: 'Memória do agente', control: { type: 'path' } },
          { key: 'paths.templates', label: 'Templates', control: { type: 'path' } },
          { key: 'paths.state', label: 'Estado local', control: { type: 'path' } },
          { key: 'paths.bugs', label: 'Relatos de defeitos', control: { type: 'path' } },
        ],
      },
    ],
  },
  {
    name: 'documents.yml',
    label: 'Documentos e marca',
    summary: 'Identidade visual, evidências embutidas e fontes dos documentos finais.',
    sections: [
      {
        title: 'Identidade',
        description: 'Aplicada nos documentos finais e neste painel. As cores dos temas são derivadas destas.',
        toggle: { label: 'Usar uma identidade própria', help: 'Desligada, vale a identidade padrão do harness.', key: 'brand.primary', clearAll: true },
        fields: [
          { key: 'brand.name', label: 'Nome do produto', control: { type: 'text' } },
          { key: 'brand.icon', label: 'Ícone', help: '.svg, .png ou .webp de até 256 KB, relativo à raiz do projeto.', control: { type: 'path' } },
          { key: 'brand.primary', label: 'Cor principal', control: { type: 'color' } },
          { key: 'brand.secondary', label: 'Cor secundária', control: { type: 'color', empty: 'Derivada da principal' } },
        ],
      },
      {
        title: 'Evidências nos documentos',
        fields: [
          { key: 'embed_max_mb', label: 'Embutir até', help: 'Imagens e vídeos maiores vão para assets/ e entram por link.', control: { type: 'number', min: 0, unit: 'MB' } },
          { key: 'text_extensions', label: 'Extensões de texto', help: 'Aparecem como bloco de código.', control: { type: 'tags', placeholder: 'Nova extensão e Enter' }, wide: true },
          { key: 'image_extensions', label: 'Extensões de imagem', control: { type: 'tags', placeholder: 'Nova extensão e Enter' }, wide: true },
          { key: 'video_extensions', label: 'Extensões de vídeo', control: { type: 'tags', placeholder: 'Nova extensão e Enter' }, wide: true },
        ],
      },
      {
        title: 'Fontes',
        description: 'Fontes do sistema, sem requisições externas.',
        fields: [
          { key: 'font.display', label: 'Títulos', control: { type: 'font' }, wide: true },
          { key: 'font.text', label: 'Texto', control: { type: 'font' }, wide: true },
          { key: 'font.body', label: 'Interface', control: { type: 'font' }, wide: true },
          { key: 'font.mono', label: 'Código', control: { type: 'font' }, wide: true },
        ],
      },
      {
        title: 'Cores dos temas',
        switcher: { label: 'Tema', options: [op('light', 'Claro'), op('dark', 'Escuro')] },
        tokens: { exclude: /^color_|^shadow_/, note: 'Derivadas da identidade pelo vostok documents brand, com o contraste mínimo de 4,5:1. Para mudar, altere a identidade.' },
        fields: [],
      },
    ],
  },
  {
    name: 'tasks.yml',
    label: 'Tarefas',
    summary: 'Execução, evidências, regressão e perfis de execução.',
    sections: [
      {
        title: 'Execução',
        fields: [
          { key: 'max_attempts', label: 'Falhas antes de bloquear', help: 'Testes que falham mais vezes que isto bloqueiam a tarefa até a sua decisão.', control: { type: 'number', min: 1, unit: 'tentativas' } },
          { key: 'max_subagents', label: 'Subagentes em paralelo numa onda', help: '', control: { type: 'range', min: 1, max: 6, unit: 'subagentes' } },
          { key: 'manual_attachment_max_mb', label: 'Limite de anexo pelo painel', help: 'Tamanho máximo de uma evidência manual; também limita a soma dos anexos de uma pergunta.', control: { type: 'number', min: 1, unit: 'MB' } },
          { key: 'evidence_baseline_timeout', label: 'Tempo-limite das evidências no baseline', help: 'Passou do tempo, a gravação é interrompida. 0 desliga o limite.', control: { type: 'number', min: 0, unit: 's' } },
          { key: 'task_types', label: 'Tipos de tarefa', help: 'Tipo do card no board.', control: { type: 'tags' } },
          { key: 'id.prefix', label: 'ID das tarefas', control: { type: 'id', digits: 'id.digits' } },
        ],
      },
      {
        title: 'Evidências no projeto',
        description: 'Cópia das evidências de cada tarefa para dentro do repositório, ao concluir.',
        toggle: { label: 'Copiar as evidências para o projeto', key: 'persist_dir' },
        fields: [
          { key: 'persist_dir', label: 'Pasta de destino', help: 'A partir da raiz do projeto, com <SPEC> e <TASK>.', control: { type: 'text', mono: true, placeholder: 'docs/evidencias/<SPEC>/<TASK>' }, wide: true },
          { key: 'persist_max_mb', label: 'Copiar arquivos até', help: 'Maiores ficam só registrados, com tamanho e SHA-256.', control: { type: 'number', min: 0, unit: 'MB' } },
          { key: 'persist_readme', label: 'Guia da pasta', control: { type: 'text', mono: true } },
        ],
      },
      {
        title: 'Regressão',
        description: 'Comparação da saída de cada comando com o baseline.',
        fields: [
          { key: 'regression_ignore', label: 'Trechos ignorados', help: 'Linhas com estes trechos saem da comparação (hora, duração).', control: { type: 'tags' }, wide: true },
          { key: 'regression_alert', label: 'Trechos de alerta', help: 'Uma linha nova com um destes trechos é regressão, mesmo sendo acréscimo.', control: { type: 'tags' }, wide: true },
        ],
      },
      {
        title: 'Perfis de execução',
        description: 'O perfil diz o que a tarefa exige do modelo. A pontuação soma os fatos; julgamento só nas combinações abaixo.',
        fields: [
          ...facts('profile_weights'),
          { key: 'profile_limits.mecanica', label: 'Mecânica até', control: { type: 'number', min: 0, unit: 'pontos' } },
          { key: 'profile_caps.files', label: 'Teto dos arquivos', control: { type: 'number', min: 0, unit: 'pontos' } },
          { key: 'profile_caps.ids', label: 'Teto dos IDs', control: { type: 'number', min: 0, unit: 'pontos' } },
          { key: 'profile_judgment', label: 'Julgamento quando a tarefa tem', control: { type: 'combos', options: FACTS }, wide: true },
          { key: 'profile_descriptions.mecanica', label: 'Descrição: mecânica', control: { type: 'textarea' }, wide: true },
          { key: 'profile_descriptions.padrao', label: 'Descrição: padrão', control: { type: 'textarea' }, wide: true },
          { key: 'profile_descriptions.julgamento', label: 'Descrição: julgamento', control: { type: 'textarea' }, wide: true },
        ],
      },
      {
        title: 'Cobertura da spec',
        description: 'Seções da spec cujos IDs as tarefas precisam cobrir.',
        switcher: { label: 'Tipo de spec', options: SPEC_TYPES, style: 'select' },
        fields: [
          { key: 'coverage_{item}', label: 'Seções obrigatórias', help: 'Só as seções que este tipo de spec tem.', control: { type: 'checklist', options: SPEC_SECTIONS, within: 'specs.yml:sections_{item}' }, wide: true },
          { key: 'coverage_optional', label: 'Seções opcionais, em todos os tipos', help: 'IDs aceitos no Cobre sem obrigação de cobrir.', control: { type: 'checklist', options: SPEC_SECTIONS }, wide: true },
        ],
      },
    ],
  },
  {
    name: 'models.yml',
    label: 'Modelos',
    summary: 'Modelo de reserva de cada perfil de execução.',
    sections: [
      {
        title: 'Modelo por perfil',
        description: 'Usado quando a ferramenta não permite listar ou escolher o modelo.',
        switcher: { label: 'Perfil', options: PROFILES },
        fields: [
          { key: 'profiles.{item}.fallback.claude-code', label: 'Reserva do harness', help: 'Atualizada a cada versão do harness.', control: { type: 'select', options: MODELS } },
          { key: 'overrides.{item}.claude-code', label: 'Modelo deste projeto', help: 'Prevalece sobre a reserva do harness.', control: { type: 'select', options: MODELS, empty: 'Usar a reserva do harness' } },
        ],
      },
    ],
  },
  {
    name: 'specs.yml',
    label: 'Specs',
    summary: 'Tipos, seções de cada tipo e documentos finais.',
    sections: [
      {
        title: 'Identificação',
        fields: [
          { key: 'id.prefix', label: 'ID das specs', control: { type: 'id', digits: 'id.digits', sequence: { file: 'sequences.yml', key: 'specs' } } },
          { key: 'types', label: 'Tipos aceitos', control: { type: 'checklist', options: SPEC_TYPES }, wide: true },
        ],
      },
      {
        title: 'Seções da spec refinada',
        description: 'Na ordem em que aparecem no documento.',
        switcher: { label: 'Tipo de spec', options: SPEC_TYPES, style: 'select' },
        fields: [{ key: 'sections_{item}', label: 'Seções', control: { type: 'checklist', options: SPEC_SECTIONS }, wide: true }],
      },
      {
        title: 'Documentos finais',
        fields: [
          { key: 'finalize_documents.executive', label: 'Executivo', control: { type: 'text', mono: true } },
          { key: 'finalize_documents.technical', label: 'Técnico', control: { type: 'text', mono: true } },
          { key: 'finalize_documents.evidence', label: 'Evidências', control: { type: 'text', mono: true } },
        ],
      },
    ],
  },
  {
    name: 'context.yml',
    label: 'Contexto',
    summary: 'O que o agente carrega sempre e em cada skill.',
    sections: [
      {
        title: 'Sempre carregado',
        description: 'Pastas e arquivos lidos no início de todo trabalho, nesta ordem.',
        fields: [{ key: 'required', label: 'Arquivos e pastas', control: { type: 'paths' }, wide: true }],
      },
      {
        title: 'Por skill',
        description: 'Carregado além do obrigatório no passo 1 de cada skill.',
        switcher: { label: 'Skill', fromKeys: 'skills', style: 'select' },
        fields: [{ key: 'skills.{item}', label: 'Arquivos da skill', control: { type: 'paths' }, wide: true }],
      },
    ],
  },
  {
    name: 'skills.yml',
    label: 'Skills',
    summary: 'Pastas, catálogo e redirecionamentos das skills.',
    sections: [
      {
        title: 'Pastas',
        fields: [
          { key: 'dirs.default', label: 'Skills do harness', control: { type: 'path' } },
          { key: 'dirs.custom', label: 'Skills do projeto', control: { type: 'path' } },
          { key: 'catalog', label: 'Catálogo', control: { type: 'path' } },
          { key: 'catalog_sections.default', label: 'Título das skills do harness', control: { type: 'text' } },
          { key: 'catalog_sections.custom', label: 'Título das skills do projeto', control: { type: 'text' } },
        ],
      },
      {
        title: 'Redirecionamentos',
        description: 'Ferramentas que recebem um atalho para cada skill nova.',
        fields: [
          {
            key: 'redirect_dirs',
            label: 'Ferramentas',
            control: {
              type: 'checklist',
              options: [
                op('.claude/skills', 'Claude Code', '.claude/skills'),
                op('.agents/skills', 'Codex', '.agents/skills'),
                op('.cursor/skills', 'Cursor', 'O Cursor já lê as outras duas: ligar duplica as skills.'),
              ],
            },
            wide: true,
          },
          { key: 'redirect_keys', label: 'Campos copiados para o atalho', control: { type: 'tags' }, wide: true },
        ],
      },
    ],
  },
  {
    name: 'memory.yml',
    label: 'Memória',
    summary: 'Tipos de memória do agente, status e categorias.',
    sections: [
      {
        title: 'Busca',
        fields: [{ key: 'search_max_results', label: 'Resultados por busca', help: 'Os demais são omitidos com um aviso.', control: { type: 'number', min: 1, unit: 'entradas' } }],
      },
      {
        title: 'Tipos de memória',
        switcher: { label: 'Tipo', options: MEMORY_TYPES },
        fields: [
          { key: '{item}.title', label: 'Título', control: { type: 'text' } },
          { key: '{item}.prefix', label: 'ID das entradas', control: { type: 'id', digits: '{item}.digits' } },
          { key: '{item}.description', label: 'Quando consultar', control: { type: 'textarea' }, wide: true },
          { key: '{item}_statuses', label: 'Status aceitos', help: 'O primeiro é o inicial.', control: { type: 'tags' }, wide: true },
          { key: '{item}.file', label: 'Arquivo', control: { type: 'text', mono: true } },
        ],
      },
      {
        title: 'Categorias dos problemas',
        fields: [{ key: 'issue_categories', label: 'Categorias', control: { type: 'tags' }, wide: true }],
      },
    ],
  },
  {
    name: 'sequences.yml',
    label: 'Numeração',
    summary: 'Próximo número de cada sequência de IDs.',
    sections: [
      {
        title: 'Próximos números',
        description: 'Mantidos pelos comandos. Mude só para pular ou reiniciar a numeração.',
        fields: [
          { key: 'specs', label: 'Próxima spec', control: { type: 'number', min: 1 } },
          { key: 'decisions', label: 'Próxima decisão', control: { type: 'number', min: 1 } },
          { key: 'issues', label: 'Próximo problema', control: { type: 'number', min: 1 } },
          { key: 'knowledge', label: 'Próximo conhecimento', control: { type: 'number', min: 1 } },
        ],
      },
    ],
  },
];

/** Nome dos tokens de cor do Atria na grade dos temas. */
export const TOKEN_LABELS: Record<string, string> = {
  bg: 'Fundo',
  bg_elevated: 'Fundo elevado',
  surface: 'Superfície',
  surface_muted: 'Superfície suave',
  surface_hover: 'Superfície ao passar',
  border: 'Borda',
  border_strong: 'Borda forte',
  primary: 'Principal',
  primary_hover: 'Principal ao passar',
  primary_soft: 'Principal suave',
  primary_on: 'Texto sobre a principal',
  accent: 'Destaque',
  accent_soft: 'Destaque suave',
  text: 'Texto',
  text_muted: 'Texto secundário',
  text_faint: 'Texto apagado',
  success: 'Sucesso',
  success_soft: 'Sucesso suave',
  warning: 'Aviso',
  warning_soft: 'Aviso suave',
  danger: 'Erro',
  danger_soft: 'Erro suave',
  info: 'Informação',
  info_soft: 'Informação suave',
  code_bg: 'Fundo de código',
  calm_from: 'Gradiente calmo: início',
  calm_to: 'Gradiente calmo: fim',
  brand_from: 'Gradiente da marca: início',
  brand_to: 'Gradiente da marca: fim',
};

/** Nome legível de uma chave sem mapeamento: `evidence_baseline_dir` vira "Evidence baseline dir". */
export function humanize(key: string): string {
  const text = key.split('.').pop()!.replace(/[_-]+/g, ' ');
  return text.charAt(0).toUpperCase() + text.slice(1);
}
