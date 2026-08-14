return { api_version = 1, id = "UNG_948", name = "Molten Reflection",
    text = "Choose a friendly minion. Summon a copy of it.", set = "UNGORO", type = "spell",
    class = "mage", rarity = "rare", spell_school = "fire", cost = 4,
    target_mode = "required", targets = function(ctx, self) return ctx:friendly_minions(self) end,
    on_play = function(ctx, self, target) ctx:summon_copy(ctx:controller(self), target) end }
