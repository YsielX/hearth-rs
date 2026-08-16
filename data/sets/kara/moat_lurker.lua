local card = {
    api_version = 1,
    id = "KAR_041",
    name = "Moat Lurker",
    text = "<b>Battlecry:</b> Destroy a minion. <b>Deathrattle:</b> Resummon it.",
    set = "KARA",
    type = "minion",
    rarity = "rare",
    cost = 6,
    attack = 3,
    health = 3,
    keywords = { "battlecry", "deathrattle" },
    target_mode = "required_if_available",
    targets = function(ctx) return ctx:minions() end,
}

function card.on_battlecry(ctx, self, target)
    if not target then return end
    ctx:set_data(self, "destroyed_minion", target)
    ctx:set_data(self, "destroyed_controller", ctx:controller(target))
    cardlib.effects.destroy(ctx, target)
end

function card.on_deathrattle(ctx, self)
    local target = ctx:get_data(self, "destroyed_minion")
    if target == 0 then return end
    local definition = ctx:card_definition(ctx:entity(target).card_id)
    ctx:summon(ctx:get_data(self, "destroyed_controller"), definition.id)
end

return card
