import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Input, Label, TextField } from "react-aria-components";
import { useForm } from "react-hook-form";
import { useIntl } from "react-intl";
import {
  commands,
  type ActionReceipt,
  type Project,
} from "@lifeos/contracts/bindings";

type ProjectFormValues = {
  title: string;
  parentProjectId: string;
  priority: string;
  startDate: string;
  targetDate: string;
};
type CommandResult<T> =
  { status: "ok"; data: T } | { status: "error"; error: unknown };

async function unwrapCommand<T>(
  command: Promise<CommandResult<T>>,
): Promise<T> {
  const result = await command;
  if (result.status === "error") throw result.error;
  return result.data;
}

export function ProjectScreen() {
  const intl = useIntl();
  const queryClient = useQueryClient();
  const [undo, setUndo] = useState<string | null>(null);
  const [receiptMessage, setReceiptMessage] = useState(
    "receipt.projectCreated",
  );
  const [editing, setEditing] = useState<Project | null>(null);
  const projects = useQuery({
    queryKey: ["projects"],
    queryFn: () => unwrapCommand(commands.listProjects()),
  });
  const archivedProjects = useQuery({
    queryKey: ["archived-projects"],
    queryFn: () => unwrapCommand(commands.listArchivedProjects()),
  });
  const trashedProjects = useQuery({
    queryKey: ["trashed-projects"],
    queryFn: () => unwrapCommand(commands.listTrashedProjects()),
  });
  const form = useForm<ProjectFormValues>({
    defaultValues: {
      title: "",
      parentProjectId: "",
      priority: "",
      startDate: "",
      targetDate: "",
    },
  });
  const create = useMutation({
    mutationFn: (values: ProjectFormValues) =>
      unwrapCommand<ActionReceipt<Project>>(
        commands.createProject({
          title: values.title,
          parentProjectId: values.parentProjectId || null,
          priority: values.priority === "" ? null : Number(values.priority),
          startDate: values.startDate || null,
          targetDate: values.targetDate || null,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      form.reset();
      setUndo(receipt.undoBatchId);
      setReceiptMessage("receipt.projectCreated");
      void queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
  const undoMutation = useMutation({
    mutationFn: (undoBatchId: string) =>
      unwrapCommand(
        commands.undoAction({ undoBatchId, operationId: crypto.randomUUID() }),
      ),
    onSuccess: () => {
      setUndo(null);
      void queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
  const update = useMutation({
    mutationFn: ({
      id,
      revision,
      values,
    }: {
      id: string;
      revision: number;
      values: ProjectFormValues;
    }) =>
      unwrapCommand<ActionReceipt<Project>>(
        commands.updateProject({
          id,
          title: values.title,
          priority: values.priority === "" ? null : Number(values.priority),
          startDate: values.startDate || null,
          targetDate: values.targetDate || null,
          expectedRevision: revision,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      setEditing(null);
      setUndo(receipt.undoBatchId);
      setReceiptMessage("receipt.projectChanged");
      void queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
  const lifecycle = useMutation({
    mutationFn: ({
      id,
      revision,
      action,
    }: {
      id: string;
      revision: number;
      action: "archive" | "trash" | "restore";
    }) =>
      unwrapCommand(
        (action === "archive"
          ? commands.archiveProject
          : action === "trash"
            ? commands.trashProject
            : commands.restoreProject)({
          id,
          expectedRevision: revision,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      setUndo(receipt.undoBatchId);
      setReceiptMessage("receipt.projectChanged");
      void Promise.all([
        queryClient.invalidateQueries({ queryKey: ["projects"] }),
        queryClient.invalidateQueries({ queryKey: ["archived-projects"] }),
        queryClient.invalidateQueries({ queryKey: ["trashed-projects"] }),
      ]);
    },
  });
  const projectById = new Map(
    projects.data?.map((project) => [project.id, project]),
  );

  return (
    <section className="canvas" aria-labelledby="projects-heading">
      <div className="eyebrow">
        {intl.formatMessage({ id: "project.eyebrow" })}
      </div>
      <div className="page-heading">
        <div>
          <h1 id="projects-heading">
            {intl.formatMessage({ id: "project.title" })}
          </h1>
          <p>{intl.formatMessage({ id: "project.description" })}</p>
        </div>
        <span
          className="metric"
          aria-label={intl.formatMessage(
            { id: "project.count" },
            { count: projects.data?.length ?? 0 },
          )}
        >
          {projects.data?.length ?? 0}
        </span>
      </div>
      <form
        className="create-card project-create-card"
        onSubmit={form.handleSubmit((values) => create.mutate(values))}
      >
        <TextField>
          <Label>{intl.formatMessage({ id: "project.name" })}</Label>
          <Input
            {...form.register("title", { required: true, maxLength: 200 })}
          />
        </TextField>
        <label className="select-field">
          {intl.formatMessage({ id: "project.parent" })}
          <select {...form.register("parentProjectId")}>
            <option value="">
              {intl.formatMessage({ id: "project.parent.none" })}
            </option>
            {projects.data?.map((project) => (
              <option key={project.id} value={project.id} dir="auto">
                {project.title}
              </option>
            ))}
          </select>
        </label>
        <TextField>
          <Label>{intl.formatMessage({ id: "project.priority" })}</Label>
          <Input
            type="number"
            min="0"
            max="100"
            {...form.register("priority", { min: 0, max: 100 })}
          />
        </TextField>
        <TextField>
          <Label>{intl.formatMessage({ id: "project.startDate" })}</Label>
          <Input type="date" {...form.register("startDate")} />
        </TextField>
        <TextField>
          <Label>{intl.formatMessage({ id: "project.targetDate" })}</Label>
          <Input type="date" {...form.register("targetDate")} />
        </TextField>
        <Button type="submit" isDisabled={create.isPending}>
          {intl.formatMessage({ id: "project.create" })}
        </Button>
      </form>
      {create.isError ? (
        <p className="form-error" role="alert">
          {intl.formatMessage({ id: "error.generic" })}
        </p>
      ) : null}
      {projects.isLoading ? (
        <p>{intl.formatMessage({ id: "common.loading" })}</p>
      ) : projects.isError ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : projects.data?.length === 0 ? (
        <div className="empty">
          <h2>{intl.formatMessage({ id: "empty.projects" })}</h2>
          <p>{intl.formatMessage({ id: "empty.projects.detail" })}</p>
        </div>
      ) : (
        <div className="area-grid">
          {projects.data?.map((project) => {
            const parent = project.parentProjectId
              ? projectById.get(project.parentProjectId)
              : undefined;
            return (
              <article className="area-card" key={project.id}>
                <span>{intl.formatMessage({ id: "entity.project" })}</span>
                <h2 dir="auto">{project.title}</h2>
                {parent ? (
                  <small>
                    {intl.formatMessage(
                      { id: "project.parentedTo" },
                      { title: <bdi>{parent.title}</bdi> },
                    )}
                  </small>
                ) : (
                  <small>
                    {intl.formatMessage({ id: "project.parent.none" })}
                  </small>
                )}
                {project.priority !== null ? (
                  <small>
                    {intl.formatMessage(
                      { id: "project.priorityValue" },
                      { priority: project.priority },
                    )}
                  </small>
                ) : null}
                {project.targetDate ? (
                  <small>
                    {intl.formatMessage({ id: "project.targetDate" })}:{" "}
                    <time dateTime={project.targetDate}>
                      {project.targetDate}
                    </time>
                  </small>
                ) : null}
                <div className="area-actions">
                  <Button onPress={() => setEditing(project)}>
                    {intl.formatMessage({ id: "project.edit" })}
                  </Button>
                  <Button
                    onPress={() =>
                      lifecycle.mutate({
                        id: project.id,
                        revision: project.revision,
                        action: "archive",
                      })
                    }
                  >
                    {intl.formatMessage({ id: "project.archive" })}
                  </Button>
                  <Button
                    onPress={() =>
                      lifecycle.mutate({
                        id: project.id,
                        revision: project.revision,
                        action: "trash",
                      })
                    }
                  >
                    {intl.formatMessage({ id: "project.trash" })}
                  </Button>
                </div>
              </article>
            );
          })}
        </div>
      )}
      <ProjectCollection
        title={intl.formatMessage({ id: "project.archivedTitle" })}
        empty={intl.formatMessage({ id: "project.archivedEmpty" })}
        projects={archivedProjects.data}
        loading={archivedProjects.isLoading}
        failed={archivedProjects.isError}
        onRestore={(project) =>
          lifecycle.mutate({
            id: project.id,
            revision: project.revision,
            action: "restore",
          })
        }
      />
      <ProjectCollection
        title={intl.formatMessage({ id: "project.trashedTitle" })}
        empty={intl.formatMessage({ id: "project.trashedEmpty" })}
        projects={trashedProjects.data}
        loading={trashedProjects.isLoading}
        failed={trashedProjects.isError}
        onRestore={(project) =>
          lifecycle.mutate({
            id: project.id,
            revision: project.revision,
            action: "restore",
          })
        }
      />
      {undo ? (
        <aside className="undo-toast" role="status">
          {intl.formatMessage({ id: receiptMessage })}
          <Button onPress={() => undoMutation.mutate(undo)}>
            {intl.formatMessage({ id: "common.undo" })}
          </Button>
        </aside>
      ) : null}
      {editing ? (
        <EditProjectForm
          project={editing}
          busy={update.isPending}
          failed={update.isError}
          onCancel={() => setEditing(null)}
          onSave={(values) =>
            update.mutate({
              id: editing.id,
              revision: editing.revision,
              values,
            })
          }
        />
      ) : null}
    </section>
  );
}

function ProjectCollection({
  title,
  empty,
  projects,
  loading,
  failed,
  onRestore,
}: {
  title: string;
  empty: string;
  projects: Project[] | undefined;
  loading: boolean;
  failed: boolean;
  onRestore: (project: Project) => void;
}) {
  const intl = useIntl();
  return (
    <section className="goal-collection" aria-label={title}>
      <h2>{title}</h2>
      {loading ? <p>{intl.formatMessage({ id: "common.loading" })}</p> : null}
      {failed ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : null}
      {projects?.length === 0 ? <p>{empty}</p> : null}
      <div className="area-grid">
        {projects?.map((project) => (
          <article className="area-card" key={project.id}>
            <h3 dir="auto">{project.title}</h3>
            <Button onPress={() => onRestore(project)}>
              {intl.formatMessage({ id: "project.restore" })}
            </Button>
          </article>
        ))}
      </div>
    </section>
  );
}

function EditProjectForm({
  project,
  busy,
  failed,
  onCancel,
  onSave,
}: {
  project: Project;
  busy: boolean;
  failed: boolean;
  onCancel: () => void;
  onSave: (values: ProjectFormValues) => void;
}) {
  const intl = useIntl();
  const form = useForm<ProjectFormValues>({
    defaultValues: {
      title: project.title,
      parentProjectId: project.parentProjectId ?? "",
      priority: project.priority?.toString() ?? "",
      startDate: project.startDate ?? "",
      targetDate: project.targetDate ?? "",
    },
  });
  return (
    <form
      className="edit-card"
      aria-labelledby="edit-project-heading"
      onSubmit={form.handleSubmit(onSave)}
    >
      <h2 id="edit-project-heading">
        {intl.formatMessage({ id: "project.editTitle" })}
      </h2>
      <TextField>
        <Label>{intl.formatMessage({ id: "project.name" })}</Label>
        <Input
          {...form.register("title", { required: true, maxLength: 200 })}
        />
      </TextField>
      <TextField>
        <Label>{intl.formatMessage({ id: "project.priority" })}</Label>
        <Input
          type="number"
          min="0"
          max="100"
          {...form.register("priority", { min: 0, max: 100 })}
        />
      </TextField>
      <TextField>
        <Label>{intl.formatMessage({ id: "project.startDate" })}</Label>
        <Input type="date" {...form.register("startDate")} />
      </TextField>
      <TextField>
        <Label>{intl.formatMessage({ id: "project.targetDate" })}</Label>
        <Input type="date" {...form.register("targetDate")} />
      </TextField>
      <div className="area-actions">
        <Button type="submit" isDisabled={busy}>
          {intl.formatMessage({ id: "common.save" })}
        </Button>
        <Button type="button" onPress={onCancel}>
          {intl.formatMessage({ id: "common.cancel" })}
        </Button>
      </div>
      {failed ? (
        <p className="form-error" role="alert">
          {intl.formatMessage({ id: "error.generic" })}
        </p>
      ) : null}
    </form>
  );
}
