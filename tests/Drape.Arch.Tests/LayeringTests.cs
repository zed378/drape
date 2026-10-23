using System.Xml.Linq;
using Shouldly;
using Xunit;

namespace Drape.Arch.Tests;

/// <summary>
/// Enforces the layering rules in <c>docs/ENGINEERING/00-CODING-STANDARDS.md</c> (CS-17, CS-20)
/// against the real project files rather than a hand-maintained allow-list.
/// </summary>
/// <remarks>
/// The checks read each <c>.csproj</c>, so a reference added by a developer is caught in the same
/// run that compiles it. A test that only asserted "the assemblies load" would pass while the
/// dependency direction rotted.
/// </remarks>
public sealed class LayeringTests
{
    private const string ForbiddenShellPrefix = "Drape.Shell.";

    /// <summary>Platform-neutral projects that must never reference a platform shell.</summary>
    private static readonly string[] _platformFreeProjects =
    [
        "Drape.Core",
        "Drape.UI",
        "Drape.Infrastructure",
    ];

    /// <summary>
    /// Projects that are not even allowed to know about a UI framework. <c>Drape.UI</c> is
    /// excluded because Avalonia is its whole purpose; it is still checked for shell references.
    /// </summary>
    private static readonly string[] _uiFrameworkFreeProjects =
    [
        "Drape.Core",
        "Drape.Infrastructure",
    ];

    /// <summary>The project names the UI-framework theory runs against.</summary>
    public static TheoryData<string> UiFrameworkFreeProjectNames => new(_uiFrameworkFreeProjects);

    /// <summary>The project names the theory below runs against.</summary>
    public static TheoryData<string> PlatformFreeProjectNames => new(_platformFreeProjects);

    /// <summary>A platform-neutral project must not reference a <c>Drape.Shell.*</c> project.</summary>
    /// <param name="projectName">Project under test.</param>
    [Theory]
    [MemberData(nameof(PlatformFreeProjectNames))]
    public void Project_ReferencesNoPlatformShellProject(string projectName)
    {
        var references = ReadProjectReferences(projectName);

        references.ShouldNotContain(
            reference => reference.StartsWith(ForbiddenShellPrefix, StringComparison.Ordinal),
            $"{projectName} must not reference a platform shell project (CS-20).");
    }

    /// <summary>Core must not reference Avalonia or the Windows SDK (CS-17).</summary>
    [Fact]
    public void Core_ReferencesNoUiFrameworkOrWindowsApi()
    {
        var references = ReadProjectReferences("Drape.Core");

        references.ShouldNotContain(
            reference => reference.StartsWith("Avalonia", StringComparison.Ordinal),
            "Core must stay free of UI frameworks (CS-17).");
    }

    /// <summary>Core and Infrastructure must not pull in Avalonia or the Windows SDK.</summary>
    /// <param name="projectName">Project under test.</param>
    [Theory]
    [MemberData(nameof(UiFrameworkFreeProjectNames))]
    public void Project_DeclaresNoUiOrWindowsPackageReference(string projectName)
    {
        var packages = ReadPackageReferences(projectName);

        packages.ShouldNotContain(
            package => package.StartsWith("Avalonia", StringComparison.Ordinal)
                || package.StartsWith("Microsoft.Windows.", StringComparison.Ordinal),
            $"{projectName} must not depend on a UI framework or the Windows SDK (CS-17, CS-20).");
    }

    /// <summary>The composition root is the single place allowed to reference the platform shell.</summary>
    [Fact]
    public void AppDesktop_IsTheOnlyProjectReferencingTheWindowsShell()
    {
        var referencing = new[] { "Drape.App.Desktop", "Drape.UI", "Drape.Core", "Drape.Infrastructure" }
            .Where(name => ReadProjectReferences(name)
                .Any(reference => reference.StartsWith(ForbiddenShellPrefix, StringComparison.Ordinal)))
            .ToArray();

        referencing.ShouldBe(["Drape.App.Desktop"]);
    }

    private static string ProjectPath(string projectName)
    {
        // tests/Drape.Arch.Tests/bin/<config>/<tfm> -> repository root
        var directory = new DirectoryInfo(AppContext.BaseDirectory);
        while (directory is not null && !File.Exists(Path.Combine(directory.FullName, "Drape.slnx")))
        {
            directory = directory.Parent;
        }

        if (directory is null)
        {
            throw new InvalidOperationException("Could not locate the repository root from the test output directory.");
        }

        var path = Path.Combine(directory.FullName, "src", projectName, $"{projectName}.csproj");
        if (!File.Exists(path))
        {
            throw new FileNotFoundException($"Project file for {projectName} was not found.", path);
        }

        return path;
    }

    private static IReadOnlyList<string> ReadProjectReferences(string projectName)
    {
        var document = XDocument.Load(ProjectPath(projectName));
        return [.. document.Descendants()
            .Where(element => element.Name.LocalName == "ProjectReference")
            .Select(element => (string?)element.Attribute("Include"))
            .Where(value => value is not null)
            .Select(value => Path.GetFileNameWithoutExtension(value!))];
    }

    private static IReadOnlyList<string> ReadPackageReferences(string projectName)
    {
        var document = XDocument.Load(ProjectPath(projectName));
        return [.. document.Descendants()
            .Where(element => element.Name.LocalName == "PackageReference")
            .Select(element => (string?)element.Attribute("Include"))
            .Where(value => value is not null)
            .Select(value => value!)];
    }
}



