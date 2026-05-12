declare module "write-excel-file/browser" {
  type Cell = {
    value?: string | number | boolean | Date | null;
    fontWeight?: "bold";
    color?: string;
    backgroundColor?: string;
    align?: "left" | "center" | "right";
    width?: number;
  };
  export default function writeXlsxFile(rows: Cell[][]): {
    toBlob: () => Promise<Blob>;
    toFile: (fileName: string) => Promise<void>;
  };
}
