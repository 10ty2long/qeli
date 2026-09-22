namespace Qeli.Shared.Vpn;

/// <summary>Parser for desktop split-route files. It accepts both qeli's one-CIDR-per-line
/// form and the common OpenVPN exports (<c>route network netmask [gateway] [metric]</c>).</summary>
internal static class RouteFileParser
{
    private const int MaxRoutes = 250_000;

    internal static IReadOnlyList<string> Load(
        IEnumerable<string> paths, CancellationToken cancellationToken, Action<string> log)
    {
        var routes = new List<string>();
        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        int files = 0;
        foreach (string rawPath in paths)
        {
            cancellationToken.ThrowIfCancellationRequested();
            string path = rawPath.Trim();
            if (path.Length == 0) continue;
            files++;
            try
            {
                foreach (string route in ParseLines(
                    File.ReadLines(path), path, cancellationToken))
                {
                    if (!seen.Add(route)) continue;
                    routes.Add(route);
                    if (routes.Count > MaxRoutes)
                        throw new InvalidDataException(
                            $"route_file set exceeds the {MaxRoutes} route safety limit");
                }
            }
            catch (OperationCanceledException) { throw; }
            catch (InvalidDataException) { throw; }
            catch (Exception e)
            {
                throw new InvalidDataException(
                    $"cannot read route_file '{path}': {e.Message}", e);
            }
        }
        if (files > 0)
            log($"Loaded {routes.Count} unique route(s) from {files} route_file source(s)");
        return routes;
    }

    internal static IReadOnlyList<string> ParseLines(
        IEnumerable<string> lines, string source = "route_file",
        CancellationToken cancellationToken = default)
    {
        var routes = new List<string>();
        var seen = new HashSet<string>(StringComparer.Ordinal);
        var batch = new List<string>(512);
        long offset = 0;
        void Flush()
        {
            cancellationToken.ThrowIfCancellationRequested();
            try
            {
                var result = Qeli.Shared.Model.ConfigCore.Policy("route_file", new { lines = batch, source, offset });
                foreach (var value in result.GetProperty("value").EnumerateArray())
                    if (seen.Add(value.GetString()!)) routes.Add(value.GetString()!);
            }
            catch (ArgumentException e) { throw new InvalidDataException(e.Message, e); }
            if (routes.Count > MaxRoutes) throw new InvalidDataException($"route_file set exceeds the {MaxRoutes} route safety limit");
            offset += batch.Count;
            batch.Clear();
        }
        foreach (var line in lines)
        {
            cancellationToken.ThrowIfCancellationRequested();
            batch.Add(line);
            if (batch.Count == 512) Flush();
        }
        if (batch.Count > 0) Flush();
        return routes;
    }
}
