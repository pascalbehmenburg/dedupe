import { ThemeProvider } from "@/components/theme-provider";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { invoke } from "@tauri-apps/api/core";
import { FileIcon, FolderIcon, FolderOpenIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "./components/ui/button";
import "./index.css";

const File = ({ path }: { path: string }) => {
  return (
    <div className="flex flex-row items-center gap-1 truncate text-pretty">
      <FileIcon className="size-5" />
      {path}
    </div>
  );
};

const DuplicateEntry = ({
  hash,
  paths,
  group_number,
}: {
  hash: string;
  paths: string[];
  group_number: number;
}) => {
  const [open, setOpen] = useState(false);
  return (
    <Collapsible open={open} onOpenChange={setOpen} key={hash}>
      <CollapsibleTrigger className="hover:bg-foreground group border-b border-border flex flex-row items-center text-start px-2 py-1 w-full">
        <div className="flex flex-col w-full">
          <p className="flex flex-row items-center gap-2 pl-1 text-pretty group-hover:text-background">
            {open ? (
              <FolderOpenIcon className="size-5 group-hover:text-background" />
            ) : (
              <FolderIcon className="size-5 group-hover:text-background" />
            )}
            {group_number} Group - {paths.length} files
          </p>
          <p className="text-muted-foreground group-hover:text-background/65 pl-8 text-sm truncate">
            [{hash}]
          </p>
        </div>
      </CollapsibleTrigger>
      {paths.map((path) => (
        <CollapsibleContent
          key={path}
          className="pl-10 hover:bg-foreground hover:text-background hover:cursor-pointer border-b border-border"
        >
          <p className="flex flex-row items-center gap-1 truncate text-pretty">
            <FileIcon className="size-5 group-hover:text-background" />
            {path}
          </p>
        </CollapsibleContent>
      ))}
    </Collapsible>
  );
};

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
    <ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
      <main className="size-full min-w-screen min-h-screen bg-background text-text antialiased">
        <div className="flex flex-row h-full">
          <div className="flex flex-col md:w-6/12 min-h-screen border border-border">
            <h1 className="text-4xl text-center py-2 border-b border-border">
              Dedupe
            </h1>
            {test &&
              Array.from(test.entries()).map(([hash, paths]) => (
                <DuplicateEntry
                  hash={hash}
                  paths={paths}
                  group_number={counter++}
                />
              ))}
            <Button
              className="rounded-none bg-background hover:bg-foreground text-text hover:text-background border-t border-border mt-auto"
              type="button"
              onClick={indexFiles}
            >
              Index Files
            </Button>
          </div>
          <div className="flex-col md:w-6/12 mx-1 my-1 pt-1 border-border border min-h-screen">
            <div className="flex flex-row h-2/12">Filename</div>
            <div className="flex flex-row bg-green-500">asdsad</div>
          </div>
        </div>
      </main>
    </ThemeProvider>
  );
}

export default App;
