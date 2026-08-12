import { useQuery } from "@tanstack/react-query";
import { useIntl } from "react-intl";
import { commands } from "@lifeos/contracts/bindings";

type CommandResult<T> =
  { status: "ok"; data: T } | { status: "error"; error: unknown };

async function unwrapCommand<T>(
  command: Promise<CommandResult<T>>,
): Promise<T> {
  const result = await command;
  if (result.status === "error") throw result.error;
  return result.data;
}

export function AuditScreen() {
  const intl = useIntl();
  const audit = useQuery({
    queryKey: ["audit-timeline"],
    queryFn: () => unwrapCommand(commands.auditEntries({ limit: 100 })),
  });

  return (
    <section className="canvas" aria-labelledby="audit-heading">
      <div className="eyebrow">
        {intl.formatMessage({ id: "audit.eyebrow" })}
      </div>
      <div className="page-heading">
        <div>
          <h1 id="audit-heading">
            {intl.formatMessage({ id: "audit.title" })}
          </h1>
          <p>{intl.formatMessage({ id: "audit.description" })}</p>
        </div>
      </div>
      {audit.isLoading ? (
        <p>{intl.formatMessage({ id: "common.loading" })}</p>
      ) : null}
      {audit.isError ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : null}
      {audit.data?.length === 0 ? (
        <div className="empty">
          <h2>{intl.formatMessage({ id: "audit.empty" })}</h2>
        </div>
      ) : null}
      <ol className="version-list audit-list">
        {audit.data?.map((entry) => (
          <li key={entry.id}>
            <strong dir="auto">{entry.action}</strong>
            <small>
              {intl.formatMessage({ id: "audit.actor" })}:{" "}
              {intl.formatMessage({ id: `actor.${entry.actorKind}` })}
            </small>
            <time dateTime={new Date(Number(entry.occurredAtMs)).toISOString()}>
              {new Intl.DateTimeFormat(intl.locale, {
                dateStyle: "medium",
                timeStyle: "short",
              }).format(Number(entry.occurredAtMs))}
            </time>
          </li>
        ))}
      </ol>
    </section>
  );
}
