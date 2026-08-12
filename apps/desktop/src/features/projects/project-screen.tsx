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
  const projects = useQuery({
    queryKey: ["projects"],
    queryFn: () => unwrapCommand(commands.listProjects()),
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
              </article>
            );
          })}
        </div>
      )}
      {undo ? (
        <aside className="undo-toast" role="status">
          {intl.formatMessage({ id: "receipt.projectCreated" })}
          <Button onPress={() => undoMutation.mutate(undo)}>
            {intl.formatMessage({ id: "common.undo" })}
          </Button>
        </aside>
      ) : null}
    </section>
  );
}
