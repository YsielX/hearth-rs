return {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_830p",
    name = "Voidform",
    text = "Deal $2 damage.\nAfter you play a card,\nrefresh this.",
    set = "ICECROWN",
    class = "priest",
    cost = 2,
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
    on_play = function(ctx, self, target) ctx:damage(target, 2) end,
    triggers = {
        {
            event = "card_played", timing = "after", active_zones = { "hero_power" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                ctx:refresh_hero_power(ctx:controller(self))
            end,
        },
    },
}
