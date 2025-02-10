import { ThemeProvider } from "@/components/theme-provider";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { invoke } from "@tauri-apps/api/core";
import {
  DatabaseZapIcon,
  FileIcon,
  FolderIcon,
  FolderOpenIcon,
} from "lucide-react";
import { useState } from "react";
import { Button } from "./components/ui/button";
import "./index.css";

type FileInfo = {
  hash: string;
  path: string;
  group_number: number;
};

const DuplicateGroup = ({
  hash,
  paths,
  group_number,
  onFileClick,
}: {
  hash: string;
  paths: string[];
  group_number: number;
  onFileClick: (hash: string, path: string, group_number: number) => void;
}) => {
  const [open, setOpen] = useState(false);
  return (
    <Collapsible open={open} onOpenChange={setOpen} key={hash}>
      <CollapsibleTrigger className="rounded-none hover:bg-foreground group border-b border-border flex flex-row items-center text-start px-2 py-1 w-full">
        <div className="flex flex-col w-full">
          <div className="flex flex-row gap-2 items-center text-pretty group-hover:text-background">
            {open ? (
              <FolderOpenIcon className="size-4 group-hover:text-background" />
            ) : (
              <FolderIcon className="size-4 group-hover:text-background" />
            )}
            <p className="whitespace-nowrap">
              {group_number} Group - {paths.length} files
            </p>
            <p className="truncate text-muted-foreground">[{hash}]</p>
          </div>
        </div>
      </CollapsibleTrigger>
      {paths.map((path) => (
        <CollapsibleContent
          key={path}
          onClick={() => onFileClick(hash, path, group_number)}
          className="pl-3 hover:bg-foreground hover:text-background hover:cursor-pointer"
        >
          <p className="pl-2 py-1 flex flex-row items-center gap-2 border-l border-border border-b truncate text-pretty">
            <FileIcon className="size-4 group-hover:text-background" />
            {path}
          </p>
        </CollapsibleContent>
      ))}
    </Collapsible>
  );
};

function App() {
  let [files, setFiles] = useState<Map<string, string[]> | null>(null);
  let [currentFile, setCurrentFile] = useState<FileInfo | null>(null);

  async function indexFiles() {
    setFiles(
      new Map(
        Object.entries(
          await invoke("index_folder", {
            path: "../test",
          })
        )
      )
    );
  }

  let group_counter = 1;
  return (
    <ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
      <main className="size-full min-w-screen min-h-screen bg-background text-text antialiased">
        <div className="flex flex-row h-full">
          <div className="flex flex-col md:w-5/12 min-h-screen border border-border">
            <h1 className="flex flex-row items-center gap-1 text-2xl px-3 py-2 border-b border-border">
              <DatabaseZapIcon className="size-6" />
              dedupe
            </h1>
            {files &&
              Array.from(files.entries()).map(([hash, paths]) => (
                <DuplicateGroup
                  hash={hash}
                  paths={paths}
                  group_number={group_counter++}
                  onFileClick={(hash, path, counter) =>
                    setCurrentFile({ hash, path, group_number: counter })
                  }
                />
              ))}
            <Button
              className="rounded-none bg-background hover:bg-foreground text-text hover:text-background border-t border-border mt-auto"
              type="button"
              onClick={indexFiles}
            >
              Index files
            </Button>
          </div>
          {currentFile && (
            <div className="flex flex-col md:w-7/12 border-border border min-h-screen">
              <h1 className="text-2xl px-4 py-2 border-b text-">
                File information
              </h1>
              <Table>
                <TableBody>
                  <TableRow>
                    <TableCell className="font-medium">Path</TableCell>
                    <TableCell>{currentFile.path}</TableCell>
                  </TableRow>

                  <TableRow>
                    <TableCell className="font-medium">Hash</TableCell>
                    <TableCell>{currentFile.hash}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">Group</TableCell>
                    <TableCell>{currentFile.group_number}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">Size</TableCell>
                    <TableCell>?</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">Content</TableCell>
                    <TableCell>?</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <div className="flex flex-row mt-auto w-full">
                <Button
                  className="rounded-none bg-background w-1/3 hover:bg-foreground text-text hover:text-background border-t border-r border-border mt-auto"
                  type="button"
                  onClick={indexFiles}
                >
                  Move
                </Button>
                <Button
                  className="rounded-none bg-secondary w-1/3 hover:bg-foreground text-text hover:text-background border-t border-r border-border mt-auto"
                  type="button"
                  variant="secondary"
                  onClick={indexFiles}
                >
                  Link
                </Button>
                <Button
                  className="rounded-none bg-destructive w-1/3 hover:bg-foreground text-text hover:text-background border-t border-border mt-auto"
                  type="button"
                  variant="destructive"
                  onClick={indexFiles}
                >
                  Delete
                </Button>
              </div>
            </div>
          )}
        </div>
      </main>
    </ThemeProvider>
  );
}

export default App;
