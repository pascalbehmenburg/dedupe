import { ThemeProvider } from "@/components/theme-provider";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Button } from "./components/ui/button";
import { Separator } from "./components/ui/separator";
import "./index.css";

function App() {
  let [test, setTest] = useState<Map<string, string[]> | null>(null);

  async function indexFiles() {
    setTest(
      new Map(
        Object.entries(
          await invoke("index_folder", {
            path: "../test",
          })
        )
      )
    );
  }

  let counter = 1;
  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <main className="size-full min-w-screen min-h-screen bg-background text-text">
        <div className="flex flex-row h-full">
          <div className="flex-col space-y-2 border-r-border py-2 bg-background w-5/12 min-h-screen">
            <h1 className="text-4xl text-center">Dedupe</h1>
            <Separator className="bg-border" />
            {test &&
              Array.from(test.entries()).map(([key, values]) => (
                <Collapsible
                  className="dark:bg-card-foreground/5 rounded-lg px-4 py-2"
                  key={key}
                >
                  <CollapsibleTrigger>
                    Duplikatgruppe {counter++}:
                  </CollapsibleTrigger>
                  {values.map((value) => (
                    <CollapsibleContent key={value} className="pl-4">
                      {value}
                    </CollapsibleContent>
                  ))}
                </Collapsible>
              ))}
            <div className="flex justify-center">
              <Button className="w-11/12" type="button" onClick={indexFiles}>
                Index Files
              </Button>
            </div>
          </div>
          <div className="flex-col bg-background/50 w-7/12 rounded-lg"></div>
        </div>
      </main>
    </ThemeProvider>
  );
}

export default App;
