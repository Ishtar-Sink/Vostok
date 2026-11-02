import { useEffect, useState } from 'react';
import { act } from './api';
import { Button } from '../ui/button/button';
import { Dialog } from '../ui/dialog/dialog';
import styles from './edit-dialog.module.css';

interface Source {
  conteudo: string;
  hash: string;
}

type State =
  | { kind: 'loading' }
  | { kind: 'editing'; hash: string }
  | { kind: 'saving'; hash: string }
  | { kind: 'conflict'; mine: string; theirs: Source }
  | { kind: 'saved'; note: string; problems: string[] }
  | { kind: 'error'; message: string; hash?: string };

/** Edição de um `.md` de `vostok/work/` com controle otimista: o hash lido vai junto da gravação, e um
 *  arquivo mudado no disco (pelo agente) volta como conflito, sem perder o texto de nenhum lado. */
export function EditDialog({ path, title, onClose }: { path: string; title: string; onClose: () => void }) {
  const [state, setState] = useState<State>({ kind: 'loading' });
  const [text, setText] = useState('');

  useEffect(() => {
    fetch(`/api/fonte?caminho=${encodeURIComponent(path)}`)
      .then((response) => (response.ok ? (response.json() as Promise<Source>) : Promise.reject(new Error(`Erro ${response.status}.`))))
      .then((source) => {
        setText(source.conteudo);
        setState({ kind: 'editing', hash: source.hash });
      })
      .catch((error: Error) => setState({ kind: 'error', message: error.message }));
  }, [path]);

  async function save(hash: string) {
    setState({ kind: 'saving', hash });
    const result = await act('PUT', `/api/arquivo?caminho=${encodeURIComponent(path)}&hash=${hash}`, text);
    if (result.status === 409) {
      setState({ kind: 'conflict', mine: text, theirs: JSON.parse(result.text) as Source });
    } else if (result.ok) {
      const saved = JSON.parse(result.text) as { registro?: string; problemas?: string[] };
      setState({ kind: 'saved', note: saved.registro ?? 'gravado', problems: saved.problemas ?? [] });
    } else {
      setState({ kind: 'error', message: result.text || `Erro ${result.status}.`, hash });
    }
  }

  const busy = state.kind === 'loading' || state.kind === 'saving';
  const actions =
    state.kind === 'conflict' ? (
      <>
        <Button variant="ghost" onClick={onClose}>Fechar</Button>
        <Button variant="secondary" onClick={() => { setText(state.theirs.conteudo); setState({ kind: 'editing', hash: state.theirs.hash }); }}>Usar a versão do disco</Button>
        <Button variant="primary" onClick={() => void save(state.theirs.hash)}>Gravar a minha por cima</Button>
      </>
    ) : state.kind === 'saved' ? (
      <Button variant="primary" onClick={onClose}>Fechar</Button>
    ) : (
      <>
        <Button variant="ghost" onClick={onClose}>Cancelar</Button>
        <Button variant="primary" disabled={busy || state.kind === 'error' && !state.hash} onClick={() => 'hash' in state && state.hash && void save(state.hash)}>
          {state.kind === 'saving' ? 'Gravando…' : 'Gravar'}
        </Button>
      </>
    );

  return (
    <Dialog open title={`Editar: ${title}`} onClose={onClose} actions={actions} size="wide">
      <p className={styles.path}>{path}</p>
      {state.kind === 'conflict' ? (
        <div className={styles.conflict} role="alert">
          <p><strong>O arquivo mudou no disco enquanto você editava.</strong> Nada foi gravado. Compare e escolha: o seu texto continua aqui.</p>
          <div className={styles.columns}>
            <label>
              <span>Seu texto</span>
              <textarea readOnly rows={12} value={state.mine} />
            </label>
            <label>
              <span>Versão no disco</span>
              <textarea readOnly rows={12} value={state.theirs.conteudo} />
            </label>
          </div>
        </div>
      ) : state.kind === 'saved' ? (
        <>
          <p className={styles.saved} role="status">Gravado. {state.note[0].toUpperCase() + state.note.slice(1)}.</p>
          {state.problems.length > 0 ? (
            <div className={styles.problems} role="alert">
              <p><strong>A validação do vostok acusou problemas.</strong> Corrija antes de aprovar ou executar:</p>
              <ul>{state.problems.map((problem) => <li key={problem}>{problem}</li>)}</ul>
            </div>
          ) : null}
        </>
      ) : (
        <>
          {state.kind === 'error' ? <p className={styles.error} role="alert">{state.message}</p> : null}
          <textarea className={styles.editor} rows={18} value={text} disabled={busy} spellCheck={false} onChange={(event) => setText(event.target.value)} aria-label={`Conteúdo de ${title}`} />
        </>
      )}
    </Dialog>
  );
}
