/// <reference path="d3.d.ts" />

var data = {
    start: 159142774583025,
    end: 159142774611944,
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
                    children: [
                    ],
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
                            children: [
                            ],
                        },
                        {
                            name: "narrow phase",
                            value: 3491,
                            start: 159142774600978,
                            end: 159142774604469,
                            children: [
                            ],
                        },
                    ],
                },
                {
                    name: "network sync",
                    value: 665,
                    start: 159142774605173,
                    end: 159142774605838,
                    children: [
                    ],
                },
            ],
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
                    children: [
                    ],
                },
                {
                    name: "draw calls",
                    value: 3460,
                    start: 159142774608124,
                    end: 159142774611584,
                    children: [
                    ],
                }
            ],
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

process(data, 0);

var min_timestamp = all_timestamps.reduce(function (a, b) {return Math.min(a, b); });

var width = document.body.clientWidth;
var barHeight = 20;

var x = d3.scale.linear().domain(all_timestamps).range([0, width]);

var axis_lines_height = 5;
var axis_text_height = 50;
var axis_height = axis_lines_height + axis_text_height;
var chart = d3.select(".chart")
.attr("width", width)
.attr("height", barHeight * max_depth + axis_height);
var ease = "sine";
var duration = 300;
var axis =
    d3.svg.axis()
      .scale(x)
      .orient("bottom")
      .tickPadding(20)
      .tickSize(axis_lines_height)
      .tickFormat(function (n) {
          return "" + ((n - min_timestamp) / 1e6);
      });

function update(selector, data, scale) {
    axis.scale(scale);
    chart.select("#axis")
         .transition().duration(duration).ease(ease)
         .call(axis);

    var bar = chart.select(selector).selectAll("g").data(out);

    var group = bar.enter().append("g");
    group.append("rect");
    group.append("text");

    bar
       .transition().duration(duration).ease(ease)
       .attr("transform", function(d) {
            var x_offset = scale(d.start);
            var y_offset = d.depth * barHeight + axis_height;
            return "translate("+ x_offset + ", " + y_offset + ")";
        }
    );

    function resize_graph(d) {
        var new_x = d3.scale.linear().domain([d.start, d.end]).range([0, width]);
        update(selector, out, new_x);
    }

    // Rectangle
    bar.select("rect")
       .on("click", resize_graph)
       .transition().duration(duration).ease(ease)
       .attr("width", function(item) {
               return scale(item.end) - scale(item.start);
        })
       .attr("height", barHeight - 1);

    // Text
    bar.select("text")
       .text(function(d) { return d.name; })
       .on("click", resize_graph)
       .transition().duration(duration).ease(ease)
       .attr("x", function(d) {
           var my_start = d.start;
           var my_end = d.end;
           var start_at_left = scale.invert(0);
           var start_at_right = scale.invert(width);

           if (my_start <= start_at_left && my_end >= start_at_right) {
               return 5 + scale(start_at_left) - scale(my_start);
           }

           return 5;
       })
       .attr("y", function(d) { return barHeight / 2; })
       .attr("dy", ".35em")
       .attr("width", function(d) {
           return scale(d.end) - scale(d.start);
       });
}

update("#bars", out, x);
