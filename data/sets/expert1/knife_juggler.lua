local card = {
    api_version = 1,
    id = "NEW1_019",
    name = "Knife Juggler",
    text = "[x]After you summon a\nminion, deal 1 damage\nto a random enemy.",
    set = "EXPERT1",
    type = "minion",
    cost = 2,
    attack = 3,
    health = 2,
}

card.triggers = {
    {
        event = "minion_summoned",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.entity ~= self
        end,
        effect = function(ctx, self, event)
            ctx:random_entity(ctx:enemy_characters(self), "throw_knife")
        end,
    },
}

card.throw_knife = function(ctx, self, target)
    ctx:damage(target, 1)
end

return card
