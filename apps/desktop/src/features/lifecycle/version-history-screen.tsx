import { useQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
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

export function VersionHistoryScreen() {
  const intl = useIntl();
  const { entityId } = useParams({ from: "/versions/$entityId" });
  const history = useQuery({
    queryKey: ["area-history", entityId],
    queryFn: () =>
      unwrapCommand(commands.areaHistory({ id: entityId, limit: 100 })),
  });

  return (
    <section className="canvas" aria-labelledby="versions-heading">
      <div className="eyebrow">
        {intl.formatMessage({ id: "history.eyebrow" })}
      </div>
      <div className="page-heading">
        <div>
          <h1 id="versions-heading">
            {intl.formatMessage({ id: "history.title" })}
          </h1>
          <p>{intl.formatMessage({ id: "history.description" })}</p>
        </div>
      </div>
      {history.isLoading ? (
        <p>{intl.formatMessage({ id: "common.loading" })}</p>
      ) : null}
      {history.isError ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : null}
      {history.data?.length === 0 ? (
        <div className="empty">
          <h2>{intl.formatMessage({ id: "history.empty" })}</h2>
        </div>
      ) : null}
      <ol className="version-list">
        {history.data?.map((version) => (
          <li key={version.revision}>
            <strong>
              {intl.formatMessage(
                { id: "entity.revision" },
                { revision: version.revision },
              )}
            </strong>
            <span dir="auto">{version.title}</span>
            <small>
              {intl.formatMessage({ id: "history.operation" })}:{" "}
              {version.operationId}
            </small>
          </li>
        ))}
      </ol>
      <Link className="back-link" to="/">
        {intl.formatMessage({ id: "history.backToAreas" })}
      </Link>
    </section>
  );
}
