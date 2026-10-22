namespace Qeli.Shared.Model;

/// <summary>Bounded, monotonic samples from existing counters. A gap is never a zero.</summary>
public sealed class TrafficHistory
{
    public const int Capacity = 120;
    public readonly record struct Rate(double Up, double Down);
    private readonly Queue<Rate> _samples = new();
    private long _tick, _up, _down;
    private DateTime? _session;
    public IReadOnlyCollection<Rate> Samples => _samples;
    public void Clear() { _samples.Clear(); _session = null; }
    public Rate? Add(long tick, long up, long down, DateTime? session, bool fresh)
    {
        if (!fresh || session is null || up < 0 || down < 0) { Clear(); return null; }
        bool reset = _session != session || up < _up || down < _down || tick <= _tick || tick - _tick > 5000;
        long elapsed = tick - _tick;
        _session = session;
        long deltaUp = reset ? 0 : up - _up, deltaDown = reset ? 0 : down - _down;
        _tick = tick; _up = up; _down = down;
        if (reset) { _samples.Clear(); return null; }
        var rate = new Rate(deltaUp * 1000.0 / elapsed, deltaDown * 1000.0 / elapsed);
        _samples.Enqueue(rate);
        if (_samples.Count > Capacity) _samples.Dequeue();
        return rate;
    }
}
