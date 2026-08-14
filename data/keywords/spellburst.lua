return {
    api_version = 1,
    module_type = "keyword",
    id = "spellburst",
    name = "Spellburst",

    -- Each card supplies only its payload. This module owns the shared timing,
    -- controller check, and one-shot consumption rule.
    required_card_hooks = { "on_spellburst" },
    triggers = {
        {
            event = "spell_cast",
            timing = "after",
            active_zones = { "board", "weapon" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                ctx:disable_keyword(self, "spellburst")
                ctx:continue_with_entity("on_spellburst", event.entity)
            end,
        },
    },
}
