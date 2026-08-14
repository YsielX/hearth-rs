return {
    api_version = 1,
    module_type = "keyword",
    id = "battlecry",
    name = "Battlecry",

    required_card_hooks = { "on_battlecry" },
    hooks = {
        on_play = function(ctx, self, target)
            if target == nil then
                ctx:continue_with("on_battlecry")
            else
                ctx:continue_with_entity("on_battlecry", target)
            end
        end,
    },
}
