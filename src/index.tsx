/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./app/App";
import { initLocaleFromStorage } from "@/shared/lib/i18n";
import "./index.css";

initLocaleFromStorage();

render(() => <App />, document.getElementById("root") as HTMLElement);
