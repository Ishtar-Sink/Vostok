import { useEffect, useMemo, useRef, useState } from 'react';
import {
  EVIDENCE_SETTINGS_URL,
  saveEvidenceSettings,
  useResource,
  type EvidenceSettings as EvidenceSettingsData,
  type SettingsValue,
} from '../../shared/api';
import { Button } from '../../ui/button/button';
import { ConfirmPopover } from '../../ui/confirm-popover/confirm-popover';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { EvidenceSettings } from './evidence-settings';
import {
  EMPTY_DRAFT,
  changePersist,
  changeValue,
  hasPending,
  localErrors,
  pendingValues,
  persistOn,
  readValues,
  shownValues,
  type Draft,
} from './settings-draft';
import { SETTING_KEYS, type DraftValue, type SettingKey } from './settings-fields';
import styles from './settings-page.module.css';
import { useLeaveGuard } from './use-leave-guard';

interface Notice {
  tone: 'success' | 'danger';
  text: string;
  conflict?: boolean;
}

const FIELD_ERROR = 'Corrija os campos marcados.';

function withoutKey<T>(record: Record<string, T>, key: string): Record<string, T> {
  const { [key]: _removed, ...rest } = record;
  return rest;
}

function generalMessage(message: string, erros: Record<string, string>): string {
  const extra = Object.entries(erros)
    .filter(([key]) => !(SETTING_KEYS as readonly string[]).includes(key))
    .map(([, text]) => text);
  return [message, ...extra].join(' ');
}

export function SettingsPage() {
  const [reload, setReload] = useState(0);
  const resource = useResource<EvidenceSettingsData>(EVIDENCE_SETTINGS_URL, reload);
  const [saved, setSaved] = useState<EvidenceSettingsData | null>(null);
  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [notice, setNotice] = useState<Notice | null>(null);
  const [saving, setSaving] = useState(false);
  const leaveRef = useRef<HTMLDivElement>(null);

  const snapshot = saved ?? (resource.kind === 'ready' ? resource.data : null);
  const base = useMemo(() => (snapshot ? readValues(snapshot.valores) : null), [snapshot]);
  const pending = base ? hasPending(base, draft) : false;
  const guard = useLeaveGuard(pending);

  useEffect(() => {
    if (guard.target !== null) leaveRef.current?.querySelector('button')?.focus();
  }, [guard.target]);

  if (!snapshot || !base) {
    if (resource.kind === 'error') {
      return (
        <ErrorState
          title="Não foi possível ler as configurações"
          description={resource.message}
          retryLabel="Tentar de novo"
          onRetry={resource.retry}
        />
      );
    }
    return <LoadingState label="Carregando as configurações" rows={8} />;
  }

  function change(key: SettingKey, value: DraftValue) {
    setDraft(changeValue(draft, key, value));
    setErrors(withoutKey(errors, key));
    if (notice?.tone === 'success') setNotice(null);
  }

  function changePersistence(on: boolean) {
    setDraft(changePersist(base!, draft, on));
    setErrors(withoutKey(errors, 'tasks.persist_dir'));
  }

  function discard() {
    setDraft(EMPTY_DRAFT);
    setErrors({});
    setNotice(null);
  }

  async function save() {
    const local = localErrors(base!, draft) as Record<string, string>;
    if (Object.keys(local).length > 0) {
      setErrors(local);
      setNotice({ tone: 'danger', text: FIELD_ERROR });
      return;
    }
    setSaving(true);
    const result = await saveEvidenceSettings(pendingValues(base!, draft) as Record<string, SettingsValue>, snapshot!.arquivos);
    setSaving(false);
    if (result.ok) {
      setSaved(result.data);
      setDraft(EMPTY_DRAFT);
      setErrors({});
      setNotice({ tone: 'success', text: 'Configurações salvas' });
    } else if (result.status === 409) {
      setNotice({ tone: 'danger', text: result.message, conflict: true });
    } else {
      const erros = result.erros ?? {};
      setErrors(erros);
      setNotice({ tone: 'danger', text: generalMessage(result.message, erros) });
    }
  }

  function reloadFromDisk() {
    setSaved(null);
    setNotice(null);
    setReload(reload + 1);
  }

  return (
    <>
      <PageHeader
        eyebrow="Projeto"
        title="Configurações"
        subtitle="As configurações voltam ao painel aos poucos, um grupo por vez. Os valores ficam em vostok/config."
      />
      <section aria-labelledby="grupo-evidencias" className={styles.group}>
        <header className={styles.groupHeader}>
          <h2 id="grupo-evidencias">Evidências</h2>
          <p>Conferência, persistência, documentos e regressão das evidências das tarefas.</p>
        </header>
        <EvidenceSettings
          values={shownValues(base, draft)}
          persist={persistOn(base, draft)}
          errors={errors}
          onChange={change}
          onPersistChange={changePersistence}
        />
        <div className={styles.actions}>
          <p className={styles.pending}>{pending ? 'Há mudanças não salvas.' : 'Nenhuma mudança.'}</p>
          {notice?.tone === 'success' ? (
            <p role="status" className={styles.saved}>{notice.text}</p>
          ) : null}
          {notice?.tone === 'danger' ? (
            <div role="alert" className={styles.alert}>
              <span>{notice.text}</span>
              {notice.conflict ? <Button size="small" onClick={reloadFromDisk}>Recarregar</Button> : null}
            </div>
          ) : null}
          <Button variant="ghost" disabled={!pending || saving} onClick={discard}>Descartar</Button>
          <Button variant="primary" disabled={!pending} loading={saving} onClick={() => void save()}>Salvar</Button>
          <div ref={leaveRef}>
            <ConfirmPopover
              open={guard.target !== null}
              className={styles.leave}
              title="Sair sem salvar?"
              description="As mudanças do grupo Evidências ainda não foram salvas e vão se perder."
              fieldLabel="Observação"
              confirmLabel="Sair sem salvar"
              cancelLabel="Continuar editando"
              tone="danger"
              onConfirm={guard.leave}
              onCancel={guard.stay}
            />
          </div>
        </div>
      </section>
    </>
  );
}
