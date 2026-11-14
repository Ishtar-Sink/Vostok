export const SETTING_KEYS = [
  'tasks.evidence_review_kinds',
  'tasks.evidence_baseline_timeout',
  'tasks.evidence_title_max',
  'tasks.persist_dir',
  'tasks.persist_max_mb',
  'tasks.persist_readme',
  'documents.embed_max_mb',
  'documents.text_extensions',
  'documents.image_extensions',
  'documents.video_extensions',
  'tasks.regression_ignore',
  'tasks.regression_alert',
] as const;

export type SettingKey = (typeof SETTING_KEYS)[number];
export type SettingValue = number | string | string[];
/** Valor em edição; `null` é o campo numérico esvaziado. */
export type DraftValue = SettingValue | null;

export interface Option {
  value: string;
  label: string;
  help: string;
}

interface SettingBase {
  key: SettingKey;
  label: string;
  help: string;
}

export type Setting =
  | (SettingBase & { kind: 'checks'; options: Option[] })
  | (SettingBase & { kind: 'number'; unit: string; min: number; max: number })
  | (SettingBase & { kind: 'text'; placeholder: string })
  | (SettingBase & { kind: 'tags'; suggestions: string[]; normalize: (text: string) => string | null });

export interface SettingsSection {
  id: 'conferencia' | 'persistencia' | 'documentos' | 'regressao';
  title: string;
  description: string;
  toggle?: { label: string; help: string };
  fields: Setting[];
}

export const REVIEW_KINDS: Option[] = [
  { value: 'automática', label: 'Automática', help: 'Saída de um comando.' },
  { value: 'automática, da verificação', label: 'Automática, da verificação', help: 'Saída de um comando rotulado na Verificação.' },
  { value: 'diff', label: 'Diff', help: 'Diferença de um arquivo entregue.' },
  { value: 'manual', label: 'Manual', help: 'Arquivo anexado por você.' },
  { value: 'print', label: 'Print', help: 'Print ou vídeo de tela.' },
];

export function normalizeExtension(text: string): string | null {
  const value = text.trim().replace(/^\.+/, '').toLowerCase();
  return /^[a-z0-9]+$/.test(value) ? value : null;
}

export function normalizeSnippet(text: string): string | null {
  return text.trim() || null;
}

export const EVIDENCE_SECTIONS: SettingsSection[] = [
  {
    id: 'conferencia',
    title: 'Conferência e geração',
    description: 'Quais evidências pedem o seu aval e os limites da gravação.',
    fields: [
      {
        kind: 'checks',
        key: 'tasks.evidence_review_kinds',
        label: 'Tipos que pedem conferência',
        help: 'Os tipos marcados só concluem a tarefa depois da sua conferência.',
        options: REVIEW_KINDS,
      },
      {
        kind: 'number',
        key: 'tasks.evidence_baseline_timeout',
        label: 'Tempo-limite das evidências no baseline',
        help: 'Passou do tempo, a gravação é interrompida. 0 desliga o limite.',
        unit: 's',
        min: 0,
        max: 3600,
      },
      {
        kind: 'number',
        key: 'tasks.evidence_title_max',
        label: 'Tamanho máximo do título',
        help: 'Limite do título curto de cada evidência.',
        unit: 'caracteres',
        min: 20,
        max: 200,
      },
    ],
  },
  {
    id: 'persistencia',
    title: 'Persistência no projeto',
    description: 'Cópia das evidências de cada tarefa para dentro do repositório, ao concluir.',
    toggle: {
      label: 'Copiar as evidências para o projeto',
      help: 'Desligada, as evidências ficam só na pasta da tarefa.',
    },
    fields: [
      {
        kind: 'text',
        key: 'tasks.persist_dir',
        label: 'Pasta de destino',
        help: 'A partir da raiz do projeto, com <SPEC> e <TASK>.',
        placeholder: 'docs/evidencias/<SPEC>/<TASK>',
      },
      {
        kind: 'number',
        key: 'tasks.persist_max_mb',
        label: 'Copiar arquivos até',
        help: 'Maiores ficam só registrados, com tamanho e SHA-256.',
        unit: 'MB',
        min: 0,
        max: 1024,
      },
      {
        kind: 'text',
        key: 'tasks.persist_readme',
        label: 'Guia da pasta',
        help: 'Nome do arquivo .md que descreve as evidências copiadas.',
        placeholder: 'README.md',
      },
    ],
  },
  {
    id: 'documentos',
    title: 'Evidências nos documentos',
    description: 'Como as evidências entram nos documentos finais da spec.',
    fields: [
      {
        kind: 'number',
        key: 'documents.embed_max_mb',
        label: 'Embutir até',
        help: 'Imagens e vídeos maiores vão para assets/ e entram por link.',
        unit: 'MB',
        min: 0,
        max: 100,
      },
      {
        kind: 'tags',
        key: 'documents.text_extensions',
        label: 'Extensões de texto',
        help: 'Aparecem como bloco de código. Sem o ponto; cada extensão numa só lista.',
        suggestions: ['out', 'txt', 'log', 'md', 'json', 'ndjson', 'csv', 'yml', 'yaml', 'xml', 'html', 'diff'],
        normalize: normalizeExtension,
      },
      {
        kind: 'tags',
        key: 'documents.image_extensions',
        label: 'Extensões de imagem',
        help: 'Embutidas até o limite; acima dele, por link.',
        suggestions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg', 'avif', 'bmp'],
        normalize: normalizeExtension,
      },
      {
        kind: 'tags',
        key: 'documents.video_extensions',
        label: 'Extensões de vídeo',
        help: 'Embutidas até o limite; acima dele, por link.',
        suggestions: ['mp4', 'webm', 'mov', 'mkv'],
        normalize: normalizeExtension,
      },
    ],
  },
  {
    id: 'regressao',
    title: 'Regressão',
    description: 'Comparação da saída de cada comando com o baseline.',
    fields: [
      {
        kind: 'tags',
        key: 'tasks.regression_ignore',
        label: 'Trechos ignorados',
        help: 'Linhas com estes trechos saem da comparação (hora, duração).',
        suggestions: ['Start at', 'Duration', 'finished in', 'Time:', 'Elapsed', 'took'],
        normalize: normalizeSnippet,
      },
      {
        kind: 'tags',
        key: 'tasks.regression_alert',
        label: 'Trechos de alerta',
        help: 'Uma linha nova com um destes trechos é regressão, mesmo sendo acréscimo.',
        suggestions: ['FAILED', 'error', 'warning', 'panicked', 'panic', 'fatal', 'Traceback'],
        normalize: normalizeSnippet,
      },
    ],
  },
];

export const DEFAULTS: Record<SettingKey, SettingValue> = {
  'tasks.evidence_review_kinds': REVIEW_KINDS.map((kind) => kind.value),
  'tasks.evidence_baseline_timeout': 120,
  'tasks.evidence_title_max': 60,
  'tasks.persist_dir': '',
  'tasks.persist_max_mb': 10,
  'tasks.persist_readme': 'README.md',
  'documents.embed_max_mb': 10,
  'documents.text_extensions': ['out', 'txt', 'log', 'md', 'json'],
  'documents.image_extensions': ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'],
  'documents.video_extensions': ['mp4', 'webm'],
  'tasks.regression_ignore': ['Start at', 'Duration', 'finished in', 'Time:'],
  'tasks.regression_alert': ['FAILED', 'error', 'warning', 'panicked'],
};
