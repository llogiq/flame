/// <reference path="d3.d.ts" />
var data = [4, 8, 15, 16, 23, 42];
var data_2 = {
    start: 159142774583025,
    end: 159142774611584,
    name: "whole program",
    children: [
        {
            name: "update",
            value: 23181,
            start: 159142774583025,
            end: 159142774606206,
            children: [
                {
                    name: "process inputs",
                    value: 4858,
                    start: 159142774593551,
                    end: 159142774598409,
                    children: []
                },
                {
                    name: "physics",
                    value: 5634,
                    start: 159142774599228,
                    end: 159142774604862,
                    children: [
                        {
                            name: "broad phase",
                            value: 773,
                            start: 159142774599897,
                            end: 159142774600670,
                            children: []
                        },
                        {
                            name: "narrow phase",
                            value: 3491,
                            start: 159142774600978,
                            end: 159142774604469,
                            children: []
                        },
                    ]
                },
                {
                    name: "network sync",
                    value: 665,
                    start: 159142774605173,
                    end: 159142774605838,
                    children: []
                },
            ]
        },
        {
            name: "render",
            value: 5445,
            start: 159142774606499,
            end: 159142774611944,
            children: [
                {
                    name: "build display lists",
                    value: 673,
                    start: 159142774607142,
                    end: 159142774607815,
                    children: []
                },
                {
                    name: "draw calls",
                    value: 3460,
                    start: 159142774608124,
                    end: 159142774611584,
                    children: []
                }
            ]
        }
    ]
};
var out = [];
var all_timestamps = [];
var max_depth = 0;
function process(span, depth) {
    span.depth = depth;
    if (depth > max_depth) {
        max_depth = depth;
    }
    all_timestamps.push(span.start);
    all_timestamps.push(span.end);
    out.push(span);
    for (var i = 0; i < span.children.length; i++) {
        process(span.children[i], depth + 1);
    }
}
process(data_2, 0);
console.log(out);
console.log(all_timestamps);
var width = 420;
var barHeight = 20;
var x = d3.scale.linear().domain(all_timestamps).range([0, width]);
var chart = d3.select(".chart")
    .attr("width", width)
    .attr("height", barHeight * max_depth);
function update(data, scale) {
    var bar = chart.selectAll("g").data(out);
    var group = bar.enter().append("g");
    group.append("rect");
    group.append("text");
    bar.attr("transform", function (d) {
        var x_offset = scale(d.start);
        var y_offset = d.depth * barHeight;
        return "translate(" + x_offset + ", " + y_offset + ")";
    });
    // Rectangle
    bar.select("rect")
        .attr("width", function (item) {
        return scale(item.end) - scale(item.start);
    })
        .attr("height", barHeight - 1)
        .on("click", function (d) {
        console.log(d);
    });
    // Text
    bar.select("text")
        .attr("y", function (d) { return barHeight / 2; })
        .attr("dy", ".35em")
        .attr("width", function (d) {
        return scale(d.end) - scale(d.start);
    })
        .text(function (d) { return d.name; });
}
update(out, x);
/*
   var x = d3.scale.linear()
   .domain([0, d3.max(data)])
   .range([0, width]);

   var chart = d3.select(".chart")
   .attr("width", width)
   .attr("height", barHeight * data.length);

   var bar = chart.selectAll("g")
   .data(data)
   .enter().append("g")
   .attr("transform", function(d, i) { return "translate(0," + i * barHeight + ")"; });

   bar.append("rect")
   .attr("width", x)
   .attr("height", barHeight - 1);

   bar.append("text")
   .attr("x", function(d) { return x(d) - 3; })
   .attr("y", barHeight / 2)
   .attr("dy", ".35em")
   .text(function(d) { return d; });
 */
