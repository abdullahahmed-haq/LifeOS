import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Dialog, DialogTrigger, Heading, Modal, ModalOverlay, TextField, Input, Label } from "react-aria-components";
import { IntlProvider, useIntl } from "react-intl";
import { useForm } from "react-hook-form";
import { commands, type ActionReceipt, type Area } from "@lifeos/contracts/bindings";
import { messages, type Locale } from "@lifeos/i18n/catalog";
import "./styles.css";

type FormValues = { title:string };
const queryClient = new QueryClient();
const operationId = () => crypto.randomUUID();
type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: unknown };

async function unwrapCommand<T>(command: Promise<CommandResult<T>>): Promise<T> {
  const result = await command;
  if (result.status === "error") throw result.error;
  return result.data;
}

function useLocale() {
  const [locale,setLocale] = useState<Locale>(() => localStorage.getItem("lifeos.locale") === "ar" ? "ar" : "en");
  const set = (next:Locale) => { localStorage.setItem("lifeos.locale",next); document.documentElement.lang=next; document.documentElement.dir=next==="ar"?"rtl":"ltr"; setLocale(next); };
  return [locale,set] as const;
}
export function App() {
  const [locale,setLocale]=useLocale();
  return <IntlProvider locale={locale} messages={messages[locale]}><QueryClientProvider client={queryClient}><Shell locale={locale} setLocale={setLocale}/></QueryClientProvider></IntlProvider>;
}
function Shell({locale,setLocale}:{locale:Locale;setLocale:(value:Locale)=>void}) {
  const intl=useIntl(); const [dialog,setDialog]=useState<string | null>(null);
  useEffect(()=>{ const handler=(event:KeyboardEvent)=>{if((event.metaKey||event.ctrlKey)&&event.key.toLowerCase()==="k"){event.preventDefault();setDialog("command");}}; window.addEventListener("keydown",handler); return()=>window.removeEventListener("keydown",handler);},[]);
  return <main className="app-shell"><aside className="sidebar"><p className="brand">{intl.formatMessage({id:"app.name"})}</p><nav><a href="#home">{intl.formatMessage({id:"navigation.home"})}</a><a href="#areas" aria-current="page">{intl.formatMessage({id:"navigation.areas"})}</a></nav><Button className="locale" onPress={()=>setLocale(locale==="en"?"ar":"en")}>{locale==="en"?"العربية":"English"}</Button></aside><section className="workspace"><header className="topbar"><Button onPress={()=>setDialog("command")}>{intl.formatMessage({id:"shell.search"})} ⌘K</Button><Button onPress={()=>setDialog("capture")}>{intl.formatMessage({id:"shell.capture"})}</Button><Button onPress={()=>setDialog("notifications")}>{intl.formatMessage({id:"shell.notifications"})}</Button><Button className="ai-trigger" onPress={()=>setDialog("ai")}>{intl.formatMessage({id:"shell.ai"})}</Button></header><AreaScreen/><DialogHost dialog={dialog} close={()=>setDialog(null)}/></section></main>;
}
function AreaScreen() {
  const intl=useIntl(); const qc=useQueryClient(); const [undo,setUndo]=useState<string | null>(null);
  const areas=useQuery({queryKey:["areas"],queryFn:()=>unwrapCommand(commands.listAreas())});
  const create=useMutation({mutationFn:(title:string)=>unwrapCommand<ActionReceipt<Area>>(commands.createArea({title,operationId:operationId()})),onSuccess:(receipt)=>{setUndo(receipt.undoBatchId); void qc.invalidateQueries({queryKey:["areas"]});}});
  const undoMutation=useMutation({mutationFn:(undoBatchId:string)=>unwrapCommand(commands.undoAction({undoBatchId,operationId:operationId()})),onSuccess:()=>{setUndo(null); void qc.invalidateQueries({queryKey:["areas"]});}});
  const form=useForm<FormValues>({defaultValues:{title:""}});
  return <section className="canvas"><div className="eyebrow">{intl.formatMessage({id:"area.eyebrow"})}</div><div className="page-heading"><div><h1>{intl.formatMessage({id:"area.title"})}</h1><p>{intl.formatMessage({id:"area.description"})}</p></div><span className="metric">{areas.data?.length ?? 0}</span></div><form className="create-card" onSubmit={form.handleSubmit(({title})=>{create.mutate(title);form.reset();})}><TextField><Label>{intl.formatMessage({id:"area.name"})}</Label><Input {...form.register("title",{required:true,maxLength:200})} /></TextField><Button type="submit" isDisabled={create.isPending}>{intl.formatMessage({id:"area.create"})}</Button></form>{areas.isLoading?<p>{intl.formatMessage({id:"common.loading"})}</p>:areas.isError?<p role="alert">{intl.formatMessage({id:"error.generic"})}</p>:areas.data?.length===0?<div className="empty"><h2>{intl.formatMessage({id:"empty.areas"})}</h2><p>{intl.formatMessage({id:"empty.areas.detail"})}</p></div>:<div className="area-grid">{areas.data?.map(area=><article className="area-card" key={area.id}><span>{intl.formatMessage({id:"entity.area"})}</span><h2 dir="auto">{area.title}</h2><small>{intl.formatMessage({id:"entity.revision"},{revision:area.revision})}</small></article>)}</div>}{undo&&<aside className="undo-toast" role="status">{intl.formatMessage({id:"receipt.areaCreated"})}<Button onPress={()=>undoMutation.mutate(undo)}>{intl.formatMessage({id:"common.undo"})}</Button></aside>}</section>;
}
function DialogHost({dialog,close}:{dialog:string|null;close:()=>void}) { const intl=useIntl(); if(!dialog)return null; const key=`dialog.${dialog}` as keyof typeof messages.en; return <DialogTrigger isOpen onOpenChange={(open)=>!open&&close()}><Button className="sr-only">{intl.formatMessage({id:"dialog.open"})}</Button><ModalOverlay className="overlay"><Modal className="modal"><Dialog><Heading slot="title">{intl.formatMessage({id:key})}</Heading><p>{dialog==="ai"?intl.formatMessage({id:"status.offline"}):intl.formatMessage({id:"dialog.placeholder"})}</p><Button onPress={close}>{intl.formatMessage({id:"common.cancel"})}</Button></Dialog></Modal></ModalOverlay></DialogTrigger>; }
const rootElement = document.getElementById("root");
if (rootElement) createRoot(rootElement).render(<StrictMode><App/></StrictMode>);
