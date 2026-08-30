return {
    api_version = 1, id = "UNG_823", name = "Envenom Weapon",
    text = "Give your weapon <b>Poisonous</b>.",
    set = "UNGORO", type = "spell", class = "rogue", rarity = "rare", spell_school = "nature", cost = 2,
    rules = {
        can_play = function(ctx, self, current)
            return current and ctx:player(ctx:controller(self)).weapon ~= nil
        end,
    },
    on_play = function(ctx, self)
        local weapon = ctx:player(ctx:controller(self)).weapon
        if weapon then cardlib.effects.grant_keyword(ctx, weapon, "poisonous") end
    end,
}
