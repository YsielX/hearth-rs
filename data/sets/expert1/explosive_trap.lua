return {
    api_version = 1,
    id = "EX1_610",
    name = "Explosive Trap",
    text = "<b>Secret:</b> When your hero is attacked, deal $2 damage to all enemies.",
    set = "EXPERT1",
    type = "spell",
    class = "hunter",
    cost = 2,
    keywords = { "secret" },
    triggers = {
        {
            event = "attack",
            timing = "before",
            active_zones = { "secret" },
            condition = function(ctx, self, event)
                return event.defender == ctx:player(ctx:controller(self)).hero
            end,
            effect = function(ctx, self, event)
                ctx:reveal_secret(self)
                cardlib.effects.damage_all(ctx, ctx:enemy_characters(self), 2)
            end,
        },
    },
}
