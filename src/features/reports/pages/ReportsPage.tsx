import { useState } from "react";
import { PDFDocument, StandardFonts, rgb } from "pdf-lib";
import { useQuery } from "@tanstack/react-query";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import writeXlsxFile from "write-excel-file/browser";
import { FileDown, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { PageHeader } from "@/components/data/PageHeader";
import { SelectableCards } from "@/components/data/SelectableCards";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { formatCurrency, todayInputValue } from "@/lib/api";
import { officeApi, type ReportsSummary } from "@/lib/office-api";
import { useAuthStore } from "@/store/auth-store";

type ReportMode = "operativo" | "resurtido";
type Format = "csv" | "xlsx" | "pdf";
type SummaryRow = { label: string; value: string };
type XlsxCell = { value?: string | number | boolean | Date | null; fontWeight?: "bold"; color?: string; backgroundColor?: string };

export function ReportsPage() {
  const sessionToken = useAuthStore((state) => state.sessionToken) ?? "";
  const [dateFrom, setDateFrom] = useState(todayInputValue());
  const [dateTo, setDateTo] = useState(todayInputValue());
  const [reportMode, setReportMode] = useState<ReportMode>("operativo");
  const [exporting, setExporting] = useState<Format | null>(null);
  const reports = useQuery({
    queryKey: ["reports", sessionToken, dateFrom, dateTo],
    queryFn: () => officeApi.getReportsSummary(sessionToken, { dateFrom, dateTo }),
    enabled: Boolean(sessionToken),
  });
  const data = reports.data;
  const summaryRows = data ? buildSummaryRows(data) : [];
  const reportTitle = reportMode === "operativo" ? "Reporte operativo" : "Reporte de resurtido de insumos";

  const exportReport = async (format: Format) => {
    if (!data) {
      toast.error("Aún no hay datos para exportar");
      return;
    }
    setExporting(format);
    try {
      const targetPath = await pickReportPath(format, reportMode);
      if (!targetPath) {
        toast.info("Exportación cancelada");
        return;
      }
      const payload =
        format === "csv"
          ? buildCsv(data, summaryRows, reportMode, dateFrom, dateTo)
          : format === "xlsx"
            ? await buildXlsxBlob(data, summaryRows, reportMode, dateFrom, dateTo)
            : await buildPdf(data, summaryRows, reportMode, dateFrom, dateTo);

      const result = await saveReport(sessionToken, payload, format, reportMode, targetPath, {
        dateFrom,
        dateTo,
        reportMode,
      });
      toast.success(`Reporte guardado: ${result.path}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setExporting(null);
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="Reportes"
        description="Reportes locales con datos vivos, resurtido separado y exportación registrada."
        actions={
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={() => reports.refetch()} disabled={reports.isFetching}>
              <RefreshCw className="h-4 w-4" />Actualizar
            </Button>
            <Button variant="outline" onClick={() => exportReport("csv")} disabled={!data || exporting !== null}>
              <FileDown className="h-4 w-4" />CSV
            </Button>
            <Button variant="outline" onClick={() => exportReport("xlsx")} disabled={!data || exporting !== null}>
              <FileDown className="h-4 w-4" />Excel
            </Button>
            <Button onClick={() => exportReport("pdf")} disabled={!data || exporting !== null}>
              <FileDown className="h-4 w-4" />PDF
            </Button>
          </div>
        }
      />

      <Card>
        <CardContent className="grid gap-4 p-4 lg:grid-cols-[1fr_180px_180px] lg:items-end">
          <div className="grid gap-2">
            <Label>Tipo de reporte</Label>
            <SelectableCards
              value={reportMode}
              onChange={(value) => setReportMode(value as ReportMode)}
              columns="sm:grid-cols-2"
              options={[
                { value: "operativo", label: "Operativo", description: "Ingresos, citas, presupuestos y saldos." },
                { value: "resurtido", label: "Resurtido", description: "Material e insumos necesarios." },
              ]}
            />
          </div>
          <div className="grid gap-2">
            <Label>Desde</Label>
            <Input type="date" value={dateFrom} onChange={(event) => setDateFrom(event.target.value)} />
          </div>
          <div className="grid gap-2">
            <Label>Hasta</Label>
            <Input type="date" value={dateTo} onChange={(event) => setDateTo(event.target.value)} />
          </div>
        </CardContent>
      </Card>

      <div className="rounded-lg border bg-card p-5">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold">{reportTitle}</h2>
            <p className="text-sm text-muted-foreground">Periodo: {dateFrom} a {dateTo}</p>
          </div>
          <div className="text-sm text-muted-foreground">
            {reports.isFetching ? "Actualizando datos..." : "Datos actualizados desde SQLite local"}
          </div>
        </div>
      </div>

      {reportMode === "operativo" ? (
        <>
          <div className="grid gap-4 md:grid-cols-3">
            {summaryRows.map((row) => (
              <Card key={row.label}>
                <CardContent className="p-5">
                  <p className="text-sm text-muted-foreground">{row.label}</p>
                  <p className="mt-2 text-2xl font-semibold">{row.value}</p>
                </CardContent>
              </Card>
            ))}
          </div>

          <div className="grid gap-4 xl:grid-cols-2">
            <ReportChart title="Ingresos por método" data={(data?.incomeByMethod ?? []).map((item) => ({ ...item, value: item.value / 100 }))} money />
            <ReportChart title="Citas por estado" data={data?.appointmentsByStatus ?? []} />
          </div>
        </>
      ) : null}

      <RestockReport data={data} expanded={reportMode === "resurtido"} />
    </div>
  );
}

function RestockReport({ data, expanded }: { data?: ReportsSummary; expanded: boolean }) {
  const rows = data?.restockItems ?? [];
  return (
    <Card>
      <CardHeader>
        <CardTitle>{expanded ? "Reporte separado de resurtido de material e insumos" : "Resurtido recomendado"}</CardTitle>
      </CardHeader>
      <CardContent className="p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Insumo</TableHead>
              <TableHead>Categoría</TableHead>
              <TableHead>Existencia</TableHead>
              <TableHead>Mínimo</TableHead>
              <TableHead>Sugerido</TableHead>
              <TableHead>Costo estimado</TableHead>
              <TableHead>Proveedor</TableHead>
              <TableHead>Caducidad</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rows.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="h-24 text-center text-muted-foreground">
                  No hay insumos pendientes de resurtir en este momento.
                </TableCell>
              </TableRow>
            ) : rows.map((item) => (
              <TableRow key={item.id}>
                <TableCell className="font-medium">{item.name}</TableCell>
                <TableCell>{item.category}</TableCell>
                <TableCell>{item.currentQuantity} {item.unit}</TableCell>
                <TableCell>{item.minimumStock} {item.unit}</TableCell>
                <TableCell>{item.suggestedQuantity} {item.unit}</TableCell>
                <TableCell>{formatCurrency(item.estimatedCostCents)}</TableCell>
                <TableCell>{item.supplierName ?? "Sin proveedor"}</TableCell>
                <TableCell>{item.expirationDate ?? "No aplica"}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function ReportChart({ title, data, money = false }: { title: string; data: { label: string; value: number }[]; money?: boolean }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
      </CardHeader>
      <CardContent className="h-72">
        {data.length === 0 ? (
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">Sin datos en el periodo seleccionado</div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data}>
              <CartesianGrid strokeDasharray="3 3" vertical={false} />
              <XAxis dataKey="label" />
              <YAxis />
              <Tooltip formatter={(value) => money ? formatCurrency(Number(value) * 100) : Number(value)} />
              <Bar dataKey="value" fill="#2563eb" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}

function buildSummaryRows(data: ReportsSummary): SummaryRow[] {
  return [
    { label: "Ingresos", value: formatCurrency(data.incomeCents) },
    { label: "Pagos", value: String(data.paymentsCount) },
    { label: "Citas", value: String(data.appointmentsCount) },
    { label: "Citas canceladas", value: String(data.cancelledAppointments) },
    { label: "Pacientes nuevos", value: String(data.newPatients) },
    { label: "Presupuestos", value: String(data.estimatesTotal) },
    { label: "Presupuestos aprobados", value: String(data.estimatesApproved) },
    { label: "Saldos pendientes", value: formatCurrency(data.pendingBalancesCents) },
    { label: "Insumos para resurtir", value: String(data.restockItems.length) },
  ];
}

function buildCsv(data: ReportsSummary, rows: SummaryRow[], mode: ReportMode, dateFrom: string, dateTo: string) {
  const sections = mode === "operativo"
    ? [
        ["DentalCare Manager", "Reporte operativo"],
        ["Periodo", `${dateFrom} a ${dateTo}`],
        [],
        ["Métrica", "Valor"],
        ...rows.map((row) => [row.label, row.value]),
        [],
        ["Ingresos por método"],
        ["Método", "Monto"],
        ...data.incomeByMethod.map((item) => [item.label, formatCurrency(item.value)]),
        [],
        ["Citas por estado"],
        ["Estado", "Cantidad"],
        ...data.appointmentsByStatus.map((item) => [item.label, String(item.value)]),
      ]
    : [
        ["DentalCare Manager", "Reporte de resurtido de material e insumos"],
        ["Periodo", `${dateFrom} a ${dateTo}`],
        [],
        ["Insumo", "Categoría", "Existencia", "Mínimo", "Sugerido", "Costo estimado", "Proveedor", "Caducidad", "Ubicación"],
        ...restockCsvRows(data),
      ];
  return sections.map((row) => row.map((cell) => `"${String(cell).replaceAll('"', '""')}"`).join(",")).join("\n");
}

async function buildXlsxBlob(data: ReportsSummary, rows: SummaryRow[], mode: ReportMode, dateFrom: string, dateTo: string) {
  const title = mode === "operativo" ? "Reporte operativo" : "Reporte de resurtido de material e insumos";
  const sheetRows: XlsxCell[][] = mode === "operativo"
    ? [
        headerRow("DentalCare Manager", title),
        [{ value: "Periodo", fontWeight: "bold" }, { value: `${dateFrom} a ${dateTo}` }],
        blankRow(),
        sectionRow("Resumen"),
        [{ value: "Métrica", fontWeight: "bold" }, { value: "Valor", fontWeight: "bold" }],
        ...rows.map((row) => [{ value: row.label }, { value: row.value }]),
        blankRow(),
        sectionRow("Ingresos por método"),
        [{ value: "Método", fontWeight: "bold" }, { value: "Monto", fontWeight: "bold" }],
        ...data.incomeByMethod.map((item) => [{ value: item.label }, { value: formatCurrency(item.value) }]),
        blankRow(),
        sectionRow("Citas por estado"),
        [{ value: "Estado", fontWeight: "bold" }, { value: "Cantidad", fontWeight: "bold" }],
        ...data.appointmentsByStatus.map((item) => [{ value: item.label }, { value: item.value }]),
      ]
    : [
        headerRow("DentalCare Manager", title),
        [{ value: "Periodo", fontWeight: "bold" }, { value: `${dateFrom} a ${dateTo}` }],
        blankRow(),
        sectionRow("Insumos necesarios"),
        restockHeader(),
        ...data.restockItems.map((item) => [
          { value: item.name },
          { value: item.category },
          { value: `${item.currentQuantity} ${item.unit}` },
          { value: `${item.minimumStock} ${item.unit}` },
          { value: `${item.suggestedQuantity} ${item.unit}` },
          { value: formatCurrency(item.estimatedCostCents) },
          { value: item.supplierName ?? "Sin proveedor" },
          { value: item.expirationDate ?? "No aplica" },
          { value: item.location ?? "Sin ubicación" },
        ]),
      ];
  return writeXlsxFile(sheetRows).toBlob();
}

async function buildPdf(data: ReportsSummary, rows: SummaryRow[], mode: ReportMode, dateFrom: string, dateTo: string) {
  const pdf = await PDFDocument.create();
  const font = await pdf.embedFont(StandardFonts.Helvetica);
  const bold = await pdf.embedFont(StandardFonts.HelveticaBold);
  let page = pdf.addPage([612, 792]);
  let y = 700;
  const title = mode === "operativo" ? "Reporte operativo" : "Reporte de resurtido de material e insumos";

  const drawHeader = () => {
    page.drawRectangle({ x: 0, y: 730, width: 612, height: 62, color: rgb(0.145, 0.388, 0.922) });
    page.drawText("DentalCare Manager", { x: 42, y: 765, size: 18, font: bold, color: rgb(1, 1, 1) });
    page.drawText(`${title} - ${dateFrom} a ${dateTo}`, { x: 42, y: 742, size: 10, font, color: rgb(0.9, 0.95, 1) });
  };
  const newPageIfNeeded = (space = 60) => {
    if (y < space) {
      page = pdf.addPage([612, 792]);
      drawHeader();
      y = 700;
    }
  };

  drawHeader();
  if (mode === "operativo") {
    drawSectionTitle(page, bold, "Resumen", y);
    y -= 24;
    rows.forEach((row) => {
      newPageIfNeeded();
      page.drawText(row.label, { x: 52, y, size: 10, font, color: rgb(0.25, 0.32, 0.42) });
      page.drawText(row.value, { x: 330, y, size: 10, font: bold, color: rgb(0.12, 0.16, 0.25) });
      y -= 18;
    });
    y -= 10;
    drawSectionTitle(page, bold, "Ingresos por método", y);
    y -= 22;
    data.incomeByMethod.forEach((item) => {
      newPageIfNeeded();
      page.drawText(item.label, { x: 52, y, size: 10, font, color: rgb(0.25, 0.32, 0.42) });
      page.drawText(formatCurrency(item.value), { x: 330, y, size: 10, font: bold, color: rgb(0.12, 0.16, 0.25) });
      y -= 18;
    });
  } else {
    drawSectionTitle(page, bold, "Insumos necesarios para resurtir", y);
    y -= 24;
    if (data.restockItems.length === 0) {
      page.drawText("No hay insumos pendientes de resurtir.", { x: 52, y, size: 10, font, color: rgb(0.35, 0.41, 0.5) });
    }
    data.restockItems.forEach((item) => {
      newPageIfNeeded(90);
      page.drawText(truncate(item.name, 68), { x: 52, y, size: 10, font: bold, color: rgb(0.12, 0.16, 0.25) });
      page.drawText(`${item.category} - existencia ${item.currentQuantity}/${item.minimumStock} ${item.unit} - sugerido ${item.suggestedQuantity} ${item.unit}`, {
        x: 52,
        y: y - 14,
        size: 9,
        font,
        color: rgb(0.35, 0.41, 0.5),
      });
      page.drawText(`Costo estimado: ${formatCurrency(item.estimatedCostCents)} - Proveedor: ${truncate(item.supplierName ?? "Sin proveedor", 45)}`, {
        x: 52,
        y: y - 28,
        size: 9,
        font,
        color: rgb(0.35, 0.41, 0.5),
      });
      page.drawText(`Caducidad: ${item.expirationDate ?? "No aplica"} - Ubicación: ${truncate(item.location ?? "Sin ubicación", 40)}`, {
        x: 52,
        y: y - 42,
        size: 9,
        font,
        color: rgb(0.35, 0.41, 0.5),
      });
      y -= 66;
    });
  }

  return pdf.save();
}

async function pickReportPath(format: Format, mode: ReportMode) {
  return saveDialog({
    title: "Guardar reporte",
    defaultPath: `${mode === "operativo" ? "reporte-operativo" : "reporte-resurtido-insumos"}.${format}`,
    filters: [
      {
        name: format === "xlsx" ? "Excel" : format.toUpperCase(),
        extensions: [format],
      },
    ],
  });
}

async function saveReport(sessionToken: string, data: string | Uint8Array | Blob, format: Format, mode: ReportMode, targetPath: string, filters: Record<string, unknown>) {
  const bytes = typeof data === "string"
    ? new TextEncoder().encode(data)
    : data instanceof Blob
      ? new Uint8Array(await data.arrayBuffer())
      : data;
  return officeApi.saveReportFile(sessionToken, {
    reportType: mode === "operativo" ? "operativo" : "resurtido_insumos",
    format,
    fileName: `${mode === "operativo" ? "reporte-operativo" : "reporte-resurtido-insumos"}.${format}`,
    targetPath,
    filtersJson: JSON.stringify(filters),
    bytes: Array.from(bytes),
  });
}

function restockCsvRows(data: ReportsSummary) {
  return data.restockItems.map((item) => [
    item.name,
    item.category,
    `${item.currentQuantity} ${item.unit}`,
    `${item.minimumStock} ${item.unit}`,
    `${item.suggestedQuantity} ${item.unit}`,
    formatCurrency(item.estimatedCostCents),
    item.supplierName ?? "Sin proveedor",
    item.expirationDate ?? "No aplica",
    item.location ?? "Sin ubicación",
  ]);
}

function headerRow(product: string, title: string): XlsxCell[] {
  return [
    { value: product, fontWeight: "bold", color: "#FFFFFF", backgroundColor: "#2563EB" },
    { value: title, fontWeight: "bold", color: "#FFFFFF", backgroundColor: "#2563EB" },
  ];
}

function sectionRow(title: string): XlsxCell[] {
  return [{ value: title, fontWeight: "bold", color: "#1F2937", backgroundColor: "#E5E7EB" }];
}

function blankRow(): XlsxCell[] {
  return [{ value: "" }];
}

function restockHeader(): XlsxCell[] {
  return ["Insumo", "Categoría", "Existencia", "Mínimo", "Sugerido", "Costo estimado", "Proveedor", "Caducidad", "Ubicación"]
    .map((value) => ({ value, fontWeight: "bold" as const, color: "#1F2937", backgroundColor: "#E5E7EB" }));
}

function drawSectionTitle(page: ReturnType<PDFDocument["addPage"]>, font: Awaited<ReturnType<PDFDocument["embedFont"]>>, text: string, y: number) {
  page.drawText(text, { x: 42, y, size: 14, font, color: rgb(0.12, 0.16, 0.25) });
}

function truncate(value: string, maxLength: number) {
  return value.length > maxLength ? `${value.slice(0, maxLength - 3)}...` : value;
}
