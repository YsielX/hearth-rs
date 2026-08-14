return {
    api_version = 1,
    module_type = "keyword",
    id = "finale",
    name = "Finale",

    required_card_hooks = { "on_finale" },
    hooks = {
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            if ctx:player(player).mana == 0 then
                ctx:continue_with("on_finale")
            end
        end,
    },
}
