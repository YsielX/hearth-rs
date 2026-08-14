return {
    api_version = 1, id = "UNG_030", name = "Binding Heal",
    text = "Restore #5 Health to a minion and your hero.",
    set = "UNGORO", type = "spell", class = "priest", rarity = "common", spell_school = "holy",
    cost = 1, target_mode = "required", targets = function(ctx, self) return ctx:minions() end,
    on_play = function(ctx, self, target)
        ctx:heal(target, 5)
        ctx:heal(ctx:player(ctx:controller(self)).hero, 5)
    end,
}
