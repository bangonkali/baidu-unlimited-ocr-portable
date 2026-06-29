[CmdletBinding()]
param(
    [string]$Configuration = "Release",
    [string]$Platform = "x64",
    [string]$PlatformToolset = "v145"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$MuPdfRoot = Join-Path $RepoRoot "thirdparty\mupdf"
$Win32Root = Join-Path $MuPdfRoot "platform\win32"
$Bin2CoffProject = Join-Path $Win32Root "bin2coff.vcxproj"
$LibMutoolProject = Join-Path $Win32Root "libmutool.vcxproj"
$MutoolProject = Join-Path $Win32Root "mutool.vcxproj"

function Ensure-MupdfSubmodule {
    if (-not (Test-Path $MutoolProject)) {
        git -C $RepoRoot submodule update --init --recursive thirdparty/mupdf
    }
    if (-not (Test-Path $MutoolProject)) {
        throw "MuPDF submodule is missing. Expected $MutoolProject"
    }
}

function Add-MsbuildElement {
    param(
        [xml]$Document,
        [System.Xml.XmlNode]$Parent,
        [string]$Name,
        [string]$Text = $null
    )
    $element = $Document.CreateElement($Name, $Document.DocumentElement.NamespaceURI)
    if ($null -ne $Text) {
        $element.InnerText = $Text
    }
    [void]$Parent.AppendChild($element)
    return $element
}

function Ensure-Bin2CoffX64Config {
    param([string]$ProjectPath)

    [xml]$project = Get-Content -LiteralPath $ProjectPath -Raw
    $namespace = New-Object System.Xml.XmlNamespaceManager($project.NameTable)
    $namespace.AddNamespace("msb", $project.DocumentElement.NamespaceURI)
    $existing = $project.SelectSingleNode("//msb:ProjectConfiguration[@Include='Release|x64']", $namespace)
    if ($existing) {
        return
    }

    $configs = $project.SelectSingleNode("//msb:ItemGroup[@Label='ProjectConfigurations']", $namespace)
    $config = Add-MsbuildElement -Document $project -Parent $configs -Name "ProjectConfiguration"
    $config.SetAttribute("Include", "Release|x64")
    Add-MsbuildElement -Document $project -Parent $config -Name "Configuration" -Text "Release" | Out-Null
    Add-MsbuildElement -Document $project -Parent $config -Name "Platform" -Text "x64" | Out-Null

    $defaultProps = $project.SelectSingleNode("//msb:Import[contains(@Project, 'Microsoft.Cpp.Default.props')]", $namespace)
    $propertyGroup = $project.CreateElement("PropertyGroup", $project.DocumentElement.NamespaceURI)
    $propertyGroup.SetAttribute("Condition", "'`$(Configuration)|`$(Platform)'=='Release|x64'")
    $propertyGroup.SetAttribute("Label", "Configuration")
    Add-MsbuildElement -Document $project -Parent $propertyGroup -Name "ConfigurationType" -Text "Application" | Out-Null
    Add-MsbuildElement -Document $project -Parent $propertyGroup -Name "PlatformToolset" -Text "v142" | Out-Null
    Add-MsbuildElement -Document $project -Parent $propertyGroup -Name "CharacterSet" -Text "Unicode" | Out-Null
    Add-MsbuildElement -Document $project -Parent $propertyGroup -Name "WholeProgramOptimization" -Text "true" | Out-Null
    [void]$project.DocumentElement.InsertAfter($propertyGroup, $defaultProps)

    $cppProps = $project.SelectSingleNode("//msb:Import[contains(@Project, 'Microsoft.Cpp.props')]", $namespace)
    $importGroup = $project.CreateElement("ImportGroup", $project.DocumentElement.NamespaceURI)
    $importGroup.SetAttribute("Condition", "'`$(Configuration)|`$(Platform)'=='Release|x64'")
    $importGroup.SetAttribute("Label", "PropertySheets")
    $userProps = $project.CreateElement("Import", $project.DocumentElement.NamespaceURI)
    $userProps.SetAttribute("Project", "`$(UserRootDir)\Microsoft.Cpp.`$(Platform).user.props")
    $userProps.SetAttribute("Condition", "exists('`$(UserRootDir)\Microsoft.Cpp.`$(Platform).user.props')")
    $userProps.SetAttribute("Label", "LocalAppDataPlatform")
    [void]$importGroup.AppendChild($userProps)
    [void]$project.DocumentElement.InsertAfter($importGroup, $cppProps)

    $userMacros = $project.SelectSingleNode("//msb:PropertyGroup[@Label='UserMacros']", $namespace)
    $paths = $project.CreateElement("PropertyGroup", $project.DocumentElement.NamespaceURI)
    $paths.SetAttribute("Condition", "'`$(Configuration)|`$(Platform)'=='Release|x64'")
    Add-MsbuildElement -Document $project -Parent $paths -Name "OutDir" -Text "`$(SolutionDir)`$(Configuration)\" | Out-Null
    Add-MsbuildElement -Document $project -Parent $paths -Name "IntDir" -Text "`$(Platform)\`$(Configuration)\`$(ProjectName)\" | Out-Null
    Add-MsbuildElement -Document $project -Parent $paths -Name "LinkIncremental" -Text "false" | Out-Null
    [void]$project.DocumentElement.InsertAfter($paths, $userMacros.NextSibling)

    $itemDefinition = $project.CreateElement("ItemDefinitionGroup", $project.DocumentElement.NamespaceURI)
    $itemDefinition.SetAttribute("Condition", "'`$(Configuration)|`$(Platform)'=='Release|x64'")
    $compile = Add-MsbuildElement -Document $project -Parent $itemDefinition -Name "ClCompile"
    Add-MsbuildElement -Document $project -Parent $compile -Name "PreprocessorDefinitions" -Text "WIN32;NDEBUG;_CONSOLE;%(PreprocessorDefinitions)" | Out-Null
    Add-MsbuildElement -Document $project -Parent $compile -Name "RuntimeLibrary" -Text "MultiThreadedDLL" | Out-Null
    Add-MsbuildElement -Document $project -Parent $compile -Name "PrecompiledHeader" | Out-Null
    Add-MsbuildElement -Document $project -Parent $compile -Name "WarningLevel" -Text "Level4" | Out-Null
    Add-MsbuildElement -Document $project -Parent $compile -Name "DebugInformationFormat" -Text "ProgramDatabase" | Out-Null
    Add-MsbuildElement -Document $project -Parent $compile -Name "DisableSpecificWarnings" -Text "4100;4200" | Out-Null
    Add-MsbuildElement -Document $project -Parent $compile -Name "MinimalRebuild" | Out-Null
    $link = Add-MsbuildElement -Document $project -Parent $itemDefinition -Name "Link"
    Add-MsbuildElement -Document $project -Parent $link -Name "GenerateDebugInformation" -Text "true" | Out-Null
    Add-MsbuildElement -Document $project -Parent $link -Name "SubSystem" -Text "Console" | Out-Null
    Add-MsbuildElement -Document $project -Parent $link -Name "OptimizeReferences" -Text "true" | Out-Null
    Add-MsbuildElement -Document $project -Parent $link -Name "EnableCOMDATFolding" -Text "true" | Out-Null
    Add-MsbuildElement -Document $project -Parent $link -Name "TargetMachine" -Text "MachineX64" | Out-Null

    $compileGroup = $project.SelectSingleNode("//msb:ItemGroup[msb:ClCompile]", $namespace)
    [void]$project.DocumentElement.InsertBefore($itemDefinition, $compileGroup)

    $settings = New-Object System.Xml.XmlWriterSettings
    $settings.Encoding = [System.Text.UTF8Encoding]::new($false)
    $settings.Indent = $true
    $writer = [System.Xml.XmlWriter]::Create($ProjectPath, $settings)
    try {
        $project.Save($writer)
    }
    finally {
        $writer.Close()
    }
}

function Disable-MissingSmartOfficeReference {
    param([string]$ProjectPath)

    $smartOfficeSource = Join-Path $MuPdfRoot "thirdparty\so\source\sodochandler.c"
    if (Test-Path $smartOfficeSource) {
        return
    }

    [xml]$project = Get-Content -LiteralPath $ProjectPath -Raw
    $namespace = New-Object System.Xml.XmlNamespaceManager($project.NameTable)
    $namespace.AddNamespace("msb", $project.DocumentElement.NamespaceURI)
    $reference = $project.SelectSingleNode("//msb:ProjectReference[@Include='sodochandler.vcxproj']", $namespace)
    if (-not $reference) {
        return
    }

    [void]$reference.ParentNode.RemoveChild($reference)
    $settings = New-Object System.Xml.XmlWriterSettings
    $settings.Encoding = [System.Text.UTF8Encoding]::new($false)
    $settings.Indent = $true
    $writer = [System.Xml.XmlWriter]::Create($ProjectPath, $settings)
    try {
        $project.Save($writer)
    }
    finally {
        $writer.Close()
    }
}

function Invoke-Msbuild {
    param([string]$ProjectPath, [string]$BuildPlatform)
    msbuild $ProjectPath `
        "/p:Configuration=$Configuration" `
        "/p:Platform=$BuildPlatform" `
        "/p:PlatformToolset=$PlatformToolset" `
        /m `
        /v:minimal
    if ($LASTEXITCODE -ne 0) {
        throw "MSBuild failed for $ProjectPath"
    }
}

Ensure-MupdfSubmodule
Ensure-Bin2CoffX64Config -ProjectPath $Bin2CoffProject
Disable-MissingSmartOfficeReference -ProjectPath $LibMutoolProject
Invoke-Msbuild -ProjectPath $Bin2CoffProject -BuildPlatform $Platform
Invoke-Msbuild -ProjectPath $MutoolProject -BuildPlatform $Platform

$MutoolExe = Join-Path $Win32Root "$Platform\$Configuration\mutool.exe"
if (-not (Test-Path $MutoolExe)) {
    throw "mutool.exe was not produced at $MutoolExe"
}
$LibMupdf = Join-Path $Win32Root "$Platform\$Configuration\libmupdf.lib"
if (-not (Test-Path $LibMupdf)) {
    throw "libmupdf.lib was not produced at $LibMupdf"
}

Write-Host "Built MuPDF static libraries for embedding into uocr-server.exe"
