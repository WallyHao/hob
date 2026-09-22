--- A representative flow for the startup benchmark: pure work only, no effects,
--- so the number is the VM, the preload and the compile.
local value = { items = { 1, 2, 3 }, name = "bench" }
local text = hob.tmpl.render("hello {name}", value)
hob.assert(text == "hello bench", "render")
local encoded = hob.json.encode({ n = #value.items })
hob.assert(#encoded > 0, "encode")
