return {
    api_version = 1,
    module_type = "keyword",
    id = "elusive",
    name = "Elusive",
    rules = {
        can_be_targeted = function(ctx, self, current, source)
            if not current or source == nil then
                return current
            end
            local source_type = ctx:entity(source).type
            return source_type ~= "spell" and source_type ~= "hero_power"
        end,
    },
}
