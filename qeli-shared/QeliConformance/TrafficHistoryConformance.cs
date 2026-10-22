using Qeli.Shared.Model;
namespace Qeli.Conformance;
internal static class TrafficHistoryConformance
{
    internal static void Run(Action<string,bool> check)
    {
        var history=new TrafficHistory(); var session=DateTime.UtcNow;
        check("traffic: first sample is baseline", history.Add(1000,100,200,session,true) is null);
        check("traffic: counter deltas per elapsed second", history.Add(2000,1100,2200,session,true) is {Up:1000,Down:2000});
        for(int i=3;i<300;i++) history.Add(i*1000,i*1000,i*2000,session,true);
        check("traffic: bounded history",history.Samples.Count==TrafficHistory.Capacity);
        check("traffic: counter reset starts new baseline", history.Add(300000,0,0,session,true) is null && history.Samples.Count==0);
        check("traffic: changed session has no spike", history.Add(301000,5000,5000,session.AddSeconds(1),true) is null);
        history.Add(302000,6000,6000,session.AddSeconds(1),true);
        check("traffic: stale observation is a gap", history.Add(303000,7000,7000,session.AddSeconds(1),false) is null && history.Samples.Count==0);
        history.Add(304000,8000,8000,session.AddSeconds(1),true);
        check("traffic: long pause cannot bridge samples", history.Add(320000,9000,9000,session.AddSeconds(1),true) is null && history.Samples.Count==0);
    }
}
