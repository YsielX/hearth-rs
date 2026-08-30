return {
    api_version = 1,
    id = "GVG_049",
    name = "Gahz'rilla",
    text = "Whenever this minion takes damage, double its Attack.",
    set = "GVG",
    type = "minion",
    class = "hunter",
    rarity = "legendary",
    cost = 7,
    attack = 6,
    health = 9,
    tags = { "beast" },
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.target == self and event.amount > 0 and ctx:entity(self).health > 0
            end,
            effect = function(ctx, self)
                cardlib.effects.buff(ctx, self, ctx:entity(self).attack, 0)
            end,
        },
    },
}
