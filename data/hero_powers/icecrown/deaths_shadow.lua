return {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_827p",
    name = "Death's Shadow",
    text = "<b>Passive</b>\nDuring your turn, add a 'Shadow Reflection' to your hand.",
    set = "ICECROWN",
    class = "rogue",
    cost = 0,
    keywords = { "passive" },
    triggers = {
        {
            event = "card_played", timing = "after", active_zones = { "hero_power" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(event.entity).card_id == "ICC_827"
            end,
            effect = function(ctx, self)
                cardlib.effects.give_card(ctx, ctx:controller(self), "ICC_827t")
            end,
        },
        {
            event = "turn_started", timing = "after", active_zones = { "hero_power" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                cardlib.effects.give_card(ctx, ctx:controller(self), "ICC_827t")
            end,
        },
        {
            event = "transformed", timing = "after", active_zones = { "hero_power" },
            condition = function(ctx, self, event)
                return event.from_card == "ICC_827t"
                    and ctx:controller(event.entity) == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                cardlib.effects.grant_keyword(ctx, event.entity, "temporary")
            end,
        },
    },
    tokens = {
        {
            id = "ICC_827t", spell_school = "shadow", name = "Shadow Reflection",
            text = "Each time you play a card, transform this into a copy of it.",
            set = "ICECROWN", type = "spell", class = "rogue", cost = 0,
            keywords = { "temporary" },
            triggers = {
                {
                    event = "card_played", timing = "before", active_zones = { "hand" },
                    condition = function(ctx, self, event)
                        return event.player == ctx:controller(self) and event.entity ~= self
                    end,
                    effect = function(ctx, self, event)
                        ctx:attach_script(self, "ICC_827t")
                        cardlib.effects.transform_preserving_scripts(ctx, self, ctx:entity(event.entity).card_id)
                    end,
                },
                {
                    event = "transformed", timing = "after", active_zones = { "hand" },
                    condition = function(ctx, self, event)
                        return event.entity == self
                    end,
                    effect = function(ctx, self)
                        cardlib.effects.grant_keyword(ctx, self, "temporary")
                    end,
                },
            },
        },
    },
}
