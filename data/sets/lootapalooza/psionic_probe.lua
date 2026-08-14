local card = {
    api_version = 1, id = "LOOT_353", name = "Psionic Probe",
    text = "Copy a spell in your opponent's deck and add it to your hand.", set = "LOOTAPALOOZA",
    type = "spell", class = "priest", rarity = "common", spell_school = "shadow", cost = 1,
}
function card.on_play(ctx, self)
    local pool = {}
    for _, entity in ipairs(ctx:deck(ctx:opponent(ctx:controller(self)))) do if ctx:entity(entity).type == "spell" then pool[#pool+1]=entity end end
    if #pool > 0 then ctx:random_entity(pool, "copy_probed_spell") end
end
function card.copy_probed_spell(ctx,self,entity) ctx:give_copy(ctx:controller(self),entity) end
return card
