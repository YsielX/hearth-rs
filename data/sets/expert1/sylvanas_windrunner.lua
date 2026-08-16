local card = {
    api_version = 1,
    id = "EX1_016",
    name = "Sylvanas Windrunner",
    text = "<b>Deathrattle:</b> Take\ncontrol of a random\nenemy minion.",
    set = "EXPERT1",
    type = "minion",
    rarity = "legendary",
    cost = 6,
    attack = 5,
    health = 5,
    tags = { "undead" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local enemies = ctx:enemy_minions(self)
    if #enemies > 0 then
        ctx:random_entity(enemies, "take_control")
    end
end

function card.take_control(ctx, self, target)
    ctx:change_controller(target, ctx:controller(self))
end

return card
