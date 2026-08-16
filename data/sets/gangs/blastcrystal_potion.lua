local card = {
    api_version = 1, id = "CFM_608", name = "Blastcrystal Potion",
    text = "Destroy a minion and one of your Mana Crystals.", set = "GANGS",
    type = "spell", class = "warlock", spell_school = "shadow", rarity = "common",
    cost = 4, target_mode = "required", targets = function(ctx) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    cardlib.effects.destroy(ctx, target)
    ctx:destroy_mana_crystals(ctx:controller(self), 1)
end
return card
