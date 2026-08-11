import { useEffect, useState } from "react";
import { Button, Input, Label, TextField } from "react-aria-components";
import { useForm } from "react-hook-form";
import { useIntl } from "react-intl";
import { usePresentation } from "../../app/presentation";

type SettingsForm = {
  timezone: string;
  weekStartsOn: "0" | "1";
};

export function GeneralSettingsScreen() {
  const intl = useIntl();
  const presentation = usePresentation();
  const [saved, setSaved] = useState(false);
  const form = useForm<SettingsForm>({
    defaultValues: {
      timezone: presentation.timezone,
      weekStartsOn: presentation.weekStartsOn === 0 ? "0" : "1",
    },
  });

  useEffect(() => {
    form.reset({
      timezone: presentation.timezone,
      weekStartsOn: presentation.weekStartsOn === 0 ? "0" : "1",
    });
  }, [form, presentation.timezone, presentation.weekStartsOn]);

  return (
    <section className="canvas" aria-labelledby="general-settings-heading">
      <div className="eyebrow">
        {intl.formatMessage({ id: "navigation.settings" })}
      </div>
      <div className="page-heading">
        <div>
          <h1 id="general-settings-heading">
            {intl.formatMessage({ id: "settings.general.title" })}
          </h1>
          <p>{intl.formatMessage({ id: "settings.general.description" })}</p>
        </div>
      </div>
      <form
        className="settings-card"
        onSubmit={form.handleSubmit(async (values) => {
          setSaved(false);
          await presentation.saveSettings({
            locale: presentation.locale,
            theme: presentation.theme,
            timezone: values.timezone.trim(),
            weekStartsOn: Number(values.weekStartsOn),
          });
          setSaved(true);
        })}
      >
        <TextField>
          <Label>{intl.formatMessage({ id: "settings.timezone" })}</Label>
          <Input
            {...form.register("timezone", { required: true, maxLength: 100 })}
          />
        </TextField>
        <label className="select-field">
          <span>{intl.formatMessage({ id: "settings.weekStart" })}</span>
          <select {...form.register("weekStartsOn")}>
            <option value="1">
              {intl.formatMessage({ id: "settings.weekStartMonday" })}
            </option>
            <option value="0">
              {intl.formatMessage({ id: "settings.weekStartSunday" })}
            </option>
          </select>
        </label>
        <Button type="submit" isDisabled={!presentation.ready}>
          {intl.formatMessage({ id: "settings.save" })}
        </Button>
      </form>
      {saved ? (
        <p role="status">{intl.formatMessage({ id: "settings.saved" })}</p>
      ) : null}
    </section>
  );
}
