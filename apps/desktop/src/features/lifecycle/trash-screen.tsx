import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "react-aria-components";
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

export function TrashScreen() {
  const intl = useIntl();
  const queryClient = useQueryClient();
  const areas = useQuery({
    queryKey: ["trashed-areas"],
    queryFn: () => unwrapCommand(commands.listTrashedAreas()),
  });
  const restore = useMutation({
    mutationFn: ({ id, revision }: { id: string; revision: number }) =>
      unwrapCommand(
        commands.restoreArea({
          id,
          expectedRevision: revision,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: () => {
      void Promise.all([
        queryClient.invalidateQueries({ queryKey: ["areas"] }),
        queryClient.invalidateQueries({ queryKey: ["trashed-areas"] }),
      ]);
    },
  });

  return (
    <section className="canvas" aria-labelledby="trash-heading">
      <div className="eyebrow">
        {intl.formatMessage({ id: "navigation.trash" })}
      </div>
      <div className="page-heading">
        <div>
          <h1 id="trash-heading">
            {intl.formatMessage({ id: "trash.title" })}
          </h1>
          <p>{intl.formatMessage({ id: "trash.description" })}</p>
        </div>
      </div>
      {areas.isLoading ? (
        <p>{intl.formatMessage({ id: "common.loading" })}</p>
      ) : null}
      {areas.isError ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : null}
      {areas.data?.length === 0 ? (
        <div className="empty">
          <h2>{intl.formatMessage({ id: "trash.empty" })}</h2>
        </div>
      ) : null}
      <div className="area-grid">
        {areas.data?.map((area) => (
          <article className="area-card" key={area.id}>
            <span>{intl.formatMessage({ id: "entity.area" })}</span>
            <h2 dir="auto">{area.title}</h2>
            <Button
              onPress={() =>
                restore.mutate({ id: area.id, revision: area.revision })
              }
            >
              {intl.formatMessage({ id: "trash.restore" })}
            </Button>
          </article>
        ))}
      </div>
    </section>
  );
}
