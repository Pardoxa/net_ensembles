(function() {var implementors = {
"net_ensembles":[["impl&lt;T:&nbsp;<a class=\"trait\" href=\"net_ensembles/traits/trait.Node.html\" title=\"trait net_ensembles::traits::Node\">Node</a>, A:&nbsp;<a class=\"trait\" href=\"net_ensembles/traits/trait.AdjContainer.html\" title=\"trait net_ensembles::traits::AdjContainer\">AdjContainer</a>&lt;T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;&amp;<a class=\"struct\" href=\"net_ensembles/generic_graph/struct.GenericGraph.html\" title=\"struct net_ensembles::generic_graph::GenericGraph\">GenericGraph</a>&lt;T, A&gt;&gt; for <a class=\"type\" href=\"net_ensembles/graph/type.Graph.html\" title=\"type net_ensembles::graph::Graph\">Graph</a>&lt;T&gt;"]],
"sampling":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/histogram/struct.AtomicHistogramFloat.html\" title=\"struct sampling::histogram::AtomicHistogramFloat\">AtomicHistogramFloat</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"sampling/histogram/struct.HistogramFloat.html\" title=\"struct sampling::histogram::HistogramFloat\">HistogramFloat</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/histogram/struct.AtomicHistogramInt.html\" title=\"struct sampling::histogram::AtomicHistogramInt\">AtomicHistogramInt</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"sampling/histogram/struct.HistogramInt.html\" title=\"struct sampling::histogram::HistogramInt\">HistogramInt</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/histogram/struct.HistogramInt.html\" title=\"struct sampling::histogram::HistogramInt\">HistogramInt</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"sampling/histogram/struct.AtomicHistogramInt.html\" title=\"struct sampling::histogram::AtomicHistogramInt\">AtomicHistogramInt</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/histogram/struct.HistogramFloat.html\" title=\"struct sampling::histogram::HistogramFloat\">HistogramFloat</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"sampling/histogram/struct.AtomicHistogramFloat.html\" title=\"struct sampling::histogram::AtomicHistogramFloat\">AtomicHistogramFloat</a>&lt;T&gt;"],["impl&lt;HistWidth, HistHeight&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/heatmap/struct.HeatmapUsizeMean.html\" title=\"struct sampling::heatmap::HeatmapUsizeMean\">HeatmapUsizeMean</a>&lt;HistWidth, HistHeight&gt;&gt; for <a class=\"type\" href=\"sampling/heatmap/type.HeatmapU.html\" title=\"type sampling::heatmap::HeatmapU\">HeatmapU</a>&lt;HistWidth, HistHeight&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/heatmap/struct.PaletteRGB.html\" title=\"struct sampling::heatmap::PaletteRGB\">PaletteRGB</a>&gt; for <a class=\"enum\" href=\"sampling/heatmap/enum.GnuplotPalette.html\" title=\"enum sampling::heatmap::GnuplotPalette\">GnuplotPalette</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/heatmap/struct.CubeHelixParameter.html\" title=\"struct sampling::heatmap::CubeHelixParameter\">CubeHelixParameter</a>&gt; for <a class=\"enum\" href=\"sampling/heatmap/enum.GnuplotPalette.html\" title=\"enum sampling::heatmap::GnuplotPalette\">GnuplotPalette</a>"],["impl&lt;HistWidth, HistHeight&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/heatmap/struct.HeatmapUsize.html\" title=\"struct sampling::heatmap::HeatmapUsize\">HeatmapUsize</a>&lt;HistWidth, HistHeight&gt;&gt; for <a class=\"struct\" href=\"sampling/heatmap/struct.HeatmapF64.html\" title=\"struct sampling::heatmap::HeatmapF64\">HeatmapF64</a>&lt;HistWidth, HistHeight&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;HistWidth: <a class=\"trait\" href=\"sampling/histogram/trait.Histogram.html\" title=\"trait sampling::histogram::Histogram\">Histogram</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;HistHeight: <a class=\"trait\" href=\"sampling/histogram/trait.Histogram.html\" title=\"trait sampling::histogram::Histogram\">Histogram</a>,</span>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"sampling/histogram/enum.HistErrors.html\" title=\"enum sampling::histogram::HistErrors\">HistErrors</a>&gt; for <a class=\"enum\" href=\"sampling/glue/enum.GlueErrors.html\" title=\"enum sampling::glue::GlueErrors\">GlueErrors</a>"],["impl&lt;Ensemble, R, Hist, Energy, S, Res&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/rewl/struct.ReplicaExchangeWangLandau.html\" title=\"struct sampling::rewl::ReplicaExchangeWangLandau\">ReplicaExchangeWangLandau</a>&lt;Ensemble, R, Hist, Energy, S, Res&gt;&gt; for <a class=\"type\" href=\"sampling/rees/type.Rees.html\" title=\"type sampling::rees::Rees\">Rees</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.66.1/std/primitive.unit.html\">()</a>, Ensemble, R, Hist, Energy, S, Res&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Hist: <a class=\"trait\" href=\"sampling/histogram/trait.Histogram.html\" title=\"trait sampling::histogram::Histogram\">Histogram</a>,</span>"],["impl&lt;R, Hist, Energy, S, Res&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"sampling/rewl/struct.RewlWalker.html\" title=\"struct sampling::rewl::RewlWalker\">RewlWalker</a>&lt;R, Hist, Energy, S, Res&gt;&gt; for <a class=\"struct\" href=\"sampling/rees/struct.ReesWalker.html\" title=\"struct sampling::rees::ReesWalker\">ReesWalker</a>&lt;R, Hist, Energy, S, Res&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Hist: <a class=\"trait\" href=\"sampling/histogram/trait.Histogram.html\" title=\"trait sampling::histogram::Histogram\">Histogram</a>,</span>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()