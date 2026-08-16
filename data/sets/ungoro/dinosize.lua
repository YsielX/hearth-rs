local card = {
    api_version = 1, id = "UNG_004", name = "Dinosize",
    text = "Set a minion's stats to 7/14.", set = "UNGORO", type = "spell", class = "paladin",
    rarity = "epic", cost = 7, target_mode = "required",
    targets = function(ctx, self) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    cardlib.effects.modify_all(ctx, { target }, {
        attack = 7, health = 14, operation = "final_set", silenciable = true, reset_damage = true,
    })
end
return card
