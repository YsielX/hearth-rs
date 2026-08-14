return {
    api_version = 1, module_type = "keyword", id = "magnetic", name = "Magnetic",
    rules = { can_magnetize = function(ctx, self, current) return true end },
}
