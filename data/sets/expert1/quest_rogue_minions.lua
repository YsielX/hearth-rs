return {
    api_version = 1,
    id = "CS2_146",
    name = "Southsea Deckhand",
    text = "Has <b>Charge</b> while you have a weapon equipped.",
    set = "EXPERT1",
    type = "minion",
    cost = 1,
    attack = 2,
    health = 1,
    tags = { "pirate" },
    keywords = { "conditional_charge" },
    tokens = {
        {
            id = "EX1_049",
            name = "Youthful Brewmaster",
            text = "<b>Battlecry:</b> Return a friendly minion from the battlefield to your hand.",
            set = "EXPERT1",
            type = "minion",
            collectible = true,
            cost = 2,
            attack = 3,
            health = 2,
            keywords = { "battlecry" },
            target_mode = "required_if_available",
            targets = function(ctx, self) return ctx:friendly_minions(self) end,
            on_battlecry = function(ctx, self, target)
                if target ~= nil then ctx:move(target, "hand") end
            end,
        },
        {
            id = "NEW1_026",
            name = "Violet Teacher",
            text = "Whenever you cast a spell, summon a 1/1 Violet Apprentice.",
            set = "EXPERT1",
            type = "minion",
            collectible = true,
            cost = 4,
            attack = 3,
            health = 5,
            triggers = {
                {
                    event = "spell_cast",
                    timing = "after",
                    active_zones = { "board" },
                    condition = function(ctx, self, event)
                        return event.player == ctx:controller(self)
                    end,
                    effect = function(ctx, self)
                        ctx:summon(ctx:controller(self), "NEW1_026t")
                    end,
                },
            },
        },
        {
            id = "NEW1_026t",
            name = "Violet Apprentice",
            set = "EXPERT1",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
        },
    },
}
